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
} = require('./constructors.js');

module.exports = walkProgram;

function walkProgram(pos, ast, visitor) {
  const enterExit = visitor.Program;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Program(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionHashbang(pos + 48, ast, visitor);
  walkVecStatement(pos + 96, ast, visitor);

  if (exit) exit(node);
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
  const visit = visitor.IdentifierName;
  if (visit !== null) visit(new IdentifierName(pos, ast));
}

function walkIdentifierReference(pos, ast, visitor) {
  const visit = visitor.IdentifierReference;
  if (visit !== null) visit(new IdentifierReference(pos, ast));
}

function walkBindingIdentifier(pos, ast, visitor) {
  const visit = visitor.BindingIdentifier;
  if (visit !== null) visit(new BindingIdentifier(pos, ast));
}

function walkLabelIdentifier(pos, ast, visitor) {
  const visit = visitor.LabelIdentifier;
  if (visit !== null) visit(new LabelIdentifier(pos, ast));
}

function walkThisExpression(pos, ast, visitor) {
  const visit = visitor.ThisExpression;
  if (visit !== null) visit(new ThisExpression(pos, ast));
}

function walkArrayExpression(pos, ast, visitor) {
  const enterExit = visitor.ArrayExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecArrayExpressionElement(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const visit = visitor.Elision;
  if (visit !== null) visit(new Elision(pos, ast));
}

function walkObjectExpression(pos, ast, visitor) {
  const enterExit = visitor.ObjectExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecObjectPropertyKind(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ObjectProperty;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TemplateLiteral;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TemplateLiteral(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 8, ast, visitor);
  walkVecExpression(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTaggedTemplateExpression(pos, ast, visitor) {
  const enterExit = visitor.TaggedTemplateExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TaggedTemplateExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkTemplateLiteral(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTemplateElement(pos, ast, visitor) {
  const visit = visitor.TemplateElement;
  if (visit !== null) visit(new TemplateElement(pos, ast));
}

function walkComputedMemberExpression(pos, ast, visitor) {
  const enterExit = visitor.ComputedMemberExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ComputedMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkStaticMemberExpression(pos, ast, visitor) {
  const enterExit = visitor.StaticMemberExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new StaticMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkIdentifierName(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkPrivateFieldExpression(pos, ast, visitor) {
  const enterExit = visitor.PrivateFieldExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PrivateFieldExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkPrivateIdentifier(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkCallExpression(pos, ast, visitor) {
  const enterExit = visitor.CallExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new CallExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkVecArgument(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkNewExpression(pos, ast, visitor) {
  const enterExit = visitor.NewExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new NewExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkVecArgument(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkMetaProperty(pos, ast, visitor) {
  const enterExit = visitor.MetaProperty;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new MetaProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);
  walkIdentifierName(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkSpreadElement(pos, ast, visitor) {
  const enterExit = visitor.SpreadElement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SpreadElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.UpdateExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new UpdateExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkSimpleAssignmentTarget(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkUnaryExpression(pos, ast, visitor) {
  const enterExit = visitor.UnaryExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new UnaryExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkBinaryExpression(pos, ast, visitor) {
  const enterExit = visitor.BinaryExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BinaryExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkPrivateInExpression(pos, ast, visitor) {
  const enterExit = visitor.PrivateInExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PrivateInExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPrivateIdentifier(pos + 8, ast, visitor);
  walkExpression(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkLogicalExpression(pos, ast, visitor) {
  const enterExit = visitor.LogicalExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new LogicalExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkConditionalExpression(pos, ast, visitor) {
  const enterExit = visitor.ConditionalExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ConditionalExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);
  walkExpression(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkAssignmentExpression(pos, ast, visitor) {
  const enterExit = visitor.AssignmentExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ArrayAssignmentTarget;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayAssignmentTarget(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecOptionAssignmentTargetMaybeDefault(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkObjectAssignmentTarget(pos, ast, visitor) {
  const enterExit = visitor.ObjectAssignmentTarget;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectAssignmentTarget(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecAssignmentTargetProperty(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.AssignmentTargetWithDefault;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetWithDefault(pos, ast);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.AssignmentTargetPropertyIdentifier;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetPropertyIdentifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierReference(pos + 8, ast, visitor);
  walkOptionExpression(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkAssignmentTargetPropertyProperty(pos, ast, visitor) {
  const enterExit = visitor.AssignmentTargetPropertyProperty;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetPropertyProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkAssignmentTargetMaybeDefault(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkSequenceExpression(pos, ast, visitor) {
  const enterExit = visitor.SequenceExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SequenceExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkSuper(pos, ast, visitor) {
  const visit = visitor.Super;
  if (visit !== null) visit(new Super(pos, ast));
}

function walkAwaitExpression(pos, ast, visitor) {
  const enterExit = visitor.AwaitExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AwaitExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkChainExpression(pos, ast, visitor) {
  const enterExit = visitor.ChainExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ChainExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkChainElement(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ParenthesizedExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ParenthesizedExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const visit = visitor.Hashbang;
  if (visit !== null) visit(new Hashbang(pos, ast));
}

function walkBlockStatement(pos, ast, visitor) {
  const enterExit = visitor.BlockStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BlockStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.VariableDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new VariableDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecVariableDeclarator(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkVariableDeclarator(pos, ast, visitor) {
  const enterExit = visitor.VariableDeclarator;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new VariableDeclarator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 8, ast, visitor);
  walkOptionExpression(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkEmptyStatement(pos, ast, visitor) {
  const visit = visitor.EmptyStatement;
  if (visit !== null) visit(new EmptyStatement(pos, ast));
}

function walkExpressionStatement(pos, ast, visitor) {
  const enterExit = visitor.ExpressionStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExpressionStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkIfStatement(pos, ast, visitor) {
  const enterExit = visitor.IfStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new IfStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkStatement(pos + 24, ast, visitor);
  walkOptionStatement(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkDoWhileStatement(pos, ast, visitor) {
  const enterExit = visitor.DoWhileStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new DoWhileStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStatement(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkWhileStatement(pos, ast, visitor) {
  const enterExit = visitor.WhileStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new WhileStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkStatement(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkForStatement(pos, ast, visitor) {
  const enterExit = visitor.ForStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionForStatementInit(pos + 8, ast, visitor);
  walkOptionExpression(pos + 24, ast, visitor);
  walkOptionExpression(pos + 40, ast, visitor);
  walkStatement(pos + 56, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ForInStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForInStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);
  walkStatement(pos + 40, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ForOfStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForOfStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);
  walkStatement(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkContinueStatement(pos, ast, visitor) {
  const enterExit = visitor.ContinueStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ContinueStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkBreakStatement(pos, ast, visitor) {
  const enterExit = visitor.BreakStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BreakStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkReturnStatement(pos, ast, visitor) {
  const enterExit = visitor.ReturnStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ReturnStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkWithStatement(pos, ast, visitor) {
  const enterExit = visitor.WithStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new WithStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkStatement(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkSwitchStatement(pos, ast, visitor) {
  const enterExit = visitor.SwitchStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SwitchStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkVecSwitchCase(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkSwitchCase(pos, ast, visitor) {
  const enterExit = visitor.SwitchCase;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SwitchCase(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitor);
  walkVecStatement(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkLabeledStatement(pos, ast, visitor) {
  const enterExit = visitor.LabeledStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new LabeledStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkLabelIdentifier(pos + 8, ast, visitor);
  walkStatement(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkThrowStatement(pos, ast, visitor) {
  const enterExit = visitor.ThrowStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ThrowStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTryStatement(pos, ast, visitor) {
  const enterExit = visitor.TryStatement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TryStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxBlockStatement(pos + 8, ast, visitor);
  walkOptionBoxCatchClause(pos + 16, ast, visitor);
  walkOptionBoxBlockStatement(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkCatchClause(pos, ast, visitor) {
  const enterExit = visitor.CatchClause;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new CatchClause(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionCatchParameter(pos + 8, ast, visitor);
  walkBoxBlockStatement(pos + 48, ast, visitor);

  if (exit) exit(node);
}

function walkCatchParameter(pos, ast, visitor) {
  walkBindingPattern(pos + 8, ast, visitor);
}

function walkDebuggerStatement(pos, ast, visitor) {
  const visit = visitor.DebuggerStatement;
  if (visit !== null) visit(new DebuggerStatement(pos, ast));
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
  const enterExit = visitor.AssignmentPattern;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 8, ast, visitor);
  walkExpression(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkObjectPattern(pos, ast, visitor) {
  const enterExit = visitor.ObjectPattern;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecBindingProperty(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkBindingProperty(pos, ast, visitor) {
  const enterExit = visitor.BindingProperty;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BindingProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkBindingPattern(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkArrayPattern(pos, ast, visitor) {
  const enterExit = visitor.ArrayPattern;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecOptionBindingPattern(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkFunction(pos, ast, visitor) {
  const enterExit = visitor.Function;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Function(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBindingIdentifier(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitor);
  walkBoxFormalParameters(pos + 56, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 64, ast, visitor);
  walkOptionBoxFunctionBody(pos + 72, ast, visitor);

  if (exit) exit(node);
}

function walkFormalParameters(pos, ast, visitor) {
  const enterExit = visitor.FormalParameters;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new FormalParameters(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecFormalParameter(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkFormalParameter(pos, ast, visitor) {
  walkVecDecorator(pos + 8, ast, visitor);
  walkBindingPattern(pos + 32, ast, visitor);
}

function walkFunctionBody(pos, ast, visitor) {
  const enterExit = visitor.FunctionBody;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new FunctionBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkArrowFunctionExpression(pos, ast, visitor) {
  const enterExit = visitor.ArrowFunctionExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrowFunctionExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 16, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);
  walkBoxFunctionBody(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkYieldExpression(pos, ast, visitor) {
  const enterExit = visitor.YieldExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new YieldExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkClass(pos, ast, visitor) {
  const enterExit = visitor.Class;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Class(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkOptionBindingIdentifier(pos + 32, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 64, ast, visitor);
  walkOptionExpression(pos + 72, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 88, ast, visitor);
  walkVecTSClassImplements(pos + 96, ast, visitor);
  walkBoxClassBody(pos + 120, ast, visitor);

  if (exit) exit(node);
}

function walkClassBody(pos, ast, visitor) {
  const enterExit = visitor.ClassBody;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ClassBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecClassElement(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.MethodDefinition;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new MethodDefinition(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkPropertyKey(pos + 32, ast, visitor);
  walkBoxFunction(pos + 48, ast, visitor);

  if (exit) exit(node);
}

function walkPropertyDefinition(pos, ast, visitor) {
  const enterExit = visitor.PropertyDefinition;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PropertyDefinition(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkPropertyKey(pos + 32, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitor);
  walkOptionExpression(pos + 56, ast, visitor);

  if (exit) exit(node);
}

function walkPrivateIdentifier(pos, ast, visitor) {
  const visit = visitor.PrivateIdentifier;
  if (visit !== null) visit(new PrivateIdentifier(pos, ast));
}

function walkStaticBlock(pos, ast, visitor) {
  const enterExit = visitor.StaticBlock;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new StaticBlock(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkAccessorProperty(pos, ast, visitor) {
  const enterExit = visitor.AccessorProperty;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AccessorProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkPropertyKey(pos + 32, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitor);
  walkOptionExpression(pos + 56, ast, visitor);

  if (exit) exit(node);
}

function walkImportExpression(pos, ast, visitor) {
  const enterExit = visitor.ImportExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkImportDeclaration(pos, ast, visitor) {
  const enterExit = visitor.ImportDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionVecImportDeclarationSpecifier(pos + 8, ast, visitor);
  walkStringLiteral(pos + 32, ast, visitor);
  walkOptionBoxWithClause(pos + 80, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ImportSpecifier;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 8, ast, visitor);
  walkBindingIdentifier(pos + 64, ast, visitor);

  if (exit) exit(node);
}

function walkImportDefaultSpecifier(pos, ast, visitor) {
  const enterExit = visitor.ImportDefaultSpecifier;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportDefaultSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkImportNamespaceSpecifier(pos, ast, visitor) {
  const enterExit = visitor.ImportNamespaceSpecifier;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportNamespaceSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkWithClause(pos, ast, visitor) {
  walkVecImportAttribute(pos + 32, ast, visitor);
}

function walkImportAttribute(pos, ast, visitor) {
  const enterExit = visitor.ImportAttribute;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkImportAttributeKey(pos + 8, ast, visitor);
  walkStringLiteral(pos + 64, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.ExportNamedDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportNamedDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionDeclaration(pos + 8, ast, visitor);
  walkVecExportSpecifier(pos + 24, ast, visitor);
  walkOptionStringLiteral(pos + 48, ast, visitor);
  walkOptionBoxWithClause(pos + 96, ast, visitor);

  if (exit) exit(node);
}

function walkExportDefaultDeclaration(pos, ast, visitor) {
  const enterExit = visitor.ExportDefaultDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportDefaultDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExportDefaultDeclarationKind(pos + 64, ast, visitor);

  if (exit) exit(node);
}

function walkExportAllDeclaration(pos, ast, visitor) {
  const enterExit = visitor.ExportAllDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportAllDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionModuleExportName(pos + 8, ast, visitor);
  walkStringLiteral(pos + 64, ast, visitor);
  walkOptionBoxWithClause(pos + 112, ast, visitor);

  if (exit) exit(node);
}

function walkExportSpecifier(pos, ast, visitor) {
  const enterExit = visitor.ExportSpecifier;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 8, ast, visitor);
  walkModuleExportName(pos + 64, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.V8IntrinsicExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new V8IntrinsicExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);
  walkVecArgument(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkBooleanLiteral(pos, ast, visitor) {
  const visit = visitor.BooleanLiteral;
  if (visit !== null) visit(new BooleanLiteral(pos, ast));
}

function walkNullLiteral(pos, ast, visitor) {
  const visit = visitor.NullLiteral;
  if (visit !== null) visit(new NullLiteral(pos, ast));
}

function walkNumericLiteral(pos, ast, visitor) {
  const visit = visitor.NumericLiteral;
  if (visit !== null) visit(new NumericLiteral(pos, ast));
}

function walkStringLiteral(pos, ast, visitor) {
  const visit = visitor.StringLiteral;
  if (visit !== null) visit(new StringLiteral(pos, ast));
}

function walkBigIntLiteral(pos, ast, visitor) {
  const visit = visitor.BigIntLiteral;
  if (visit !== null) visit(new BigIntLiteral(pos, ast));
}

function walkRegExpLiteral(pos, ast, visitor) {
  const visit = visitor.RegExpLiteral;
  if (visit !== null) visit(new RegExpLiteral(pos, ast));
}

function walkJSXElement(pos, ast, visitor) {
  const enterExit = visitor.JSXElement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxJSXOpeningElement(pos + 8, ast, visitor);
  walkVecJSXChild(pos + 16, ast, visitor);
  walkOptionBoxJSXClosingElement(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkJSXOpeningElement(pos, ast, visitor) {
  const enterExit = visitor.JSXOpeningElement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXOpeningElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkVecJSXAttributeItem(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkJSXClosingElement(pos, ast, visitor) {
  const enterExit = visitor.JSXClosingElement;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXClosingElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkJSXFragment(pos, ast, visitor) {
  const enterExit = visitor.JSXFragment;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXFragment(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXOpeningFragment(pos + 8, ast, visitor);
  walkVecJSXChild(pos + 16, ast, visitor);
  walkJSXClosingFragment(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkJSXOpeningFragment(pos, ast, visitor) {
  const visit = visitor.JSXOpeningFragment;
  if (visit !== null) visit(new JSXOpeningFragment(pos, ast));
}

function walkJSXClosingFragment(pos, ast, visitor) {
  const visit = visitor.JSXClosingFragment;
  if (visit !== null) visit(new JSXClosingFragment(pos, ast));
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
  const enterExit = visitor.JSXNamespacedName;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXNamespacedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXIdentifier(pos + 8, ast, visitor);
  walkJSXIdentifier(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkJSXMemberExpression(pos, ast, visitor) {
  const enterExit = visitor.JSXMemberExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXMemberExpressionObject(pos + 8, ast, visitor);
  walkJSXIdentifier(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.JSXExpressionContainer;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXExpressionContainer(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const visit = visitor.JSXEmptyExpression;
  if (visit !== null) visit(new JSXEmptyExpression(pos, ast));
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
  const enterExit = visitor.JSXAttribute;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXAttributeName(pos + 8, ast, visitor);
  walkOptionJSXAttributeValue(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkJSXSpreadAttribute(pos, ast, visitor) {
  const enterExit = visitor.JSXSpreadAttribute;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXSpreadAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const visit = visitor.JSXIdentifier;
  if (visit !== null) visit(new JSXIdentifier(pos, ast));
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
  const enterExit = visitor.JSXSpreadChild;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXSpreadChild(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkJSXText(pos, ast, visitor) {
  const visit = visitor.JSXText;
  if (visit !== null) visit(new JSXText(pos, ast));
}

function walkTSEnumDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSEnumDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkTSEnumBody(pos + 40, ast, visitor);

  if (exit) exit(node);
}

function walkTSEnumBody(pos, ast, visitor) {
  const enterExit = visitor.TSEnumBody;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSEnumMember(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSEnumMember(pos, ast, visitor) {
  const enterExit = visitor.TSEnumMember;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumMember(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSEnumMemberName(pos + 8, ast, visitor);
  walkOptionExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSTypeAnnotation;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAnnotation(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSLiteralType(pos, ast, visitor) {
  const enterExit = visitor.TSLiteralType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSLiteralType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSLiteral(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSConditionalType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConditionalType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);
  walkTSType(pos + 40, ast, visitor);
  walkTSType(pos + 56, ast, visitor);

  if (exit) exit(node);
}

function walkTSUnionType(pos, ast, visitor) {
  const enterExit = visitor.TSUnionType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSUnionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSIntersectionType(pos, ast, visitor) {
  const enterExit = visitor.TSIntersectionType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIntersectionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSParenthesizedType(pos, ast, visitor) {
  const enterExit = visitor.TSParenthesizedType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSParenthesizedType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeOperator(pos, ast, visitor) {
  const enterExit = visitor.TSTypeOperator;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeOperator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSArrayType(pos, ast, visitor) {
  const enterExit = visitor.TSArrayType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSArrayType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSIndexedAccessType(pos, ast, visitor) {
  const enterExit = visitor.TSIndexedAccessType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexedAccessType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSTupleType(pos, ast, visitor) {
  const enterExit = visitor.TSTupleType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTupleType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSTupleElement(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSNamedTupleMember(pos, ast, visitor) {
  const enterExit = visitor.TSNamedTupleMember;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamedTupleMember(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);
  walkTSTupleElement(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSOptionalType(pos, ast, visitor) {
  const enterExit = visitor.TSOptionalType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSOptionalType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSRestType(pos, ast, visitor) {
  const enterExit = visitor.TSRestType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSRestType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
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
  const visit = visitor.TSAnyKeyword;
  if (visit !== null) visit(new TSAnyKeyword(pos, ast));
}

function walkTSStringKeyword(pos, ast, visitor) {
  const visit = visitor.TSStringKeyword;
  if (visit !== null) visit(new TSStringKeyword(pos, ast));
}

function walkTSBooleanKeyword(pos, ast, visitor) {
  const visit = visitor.TSBooleanKeyword;
  if (visit !== null) visit(new TSBooleanKeyword(pos, ast));
}

function walkTSNumberKeyword(pos, ast, visitor) {
  const visit = visitor.TSNumberKeyword;
  if (visit !== null) visit(new TSNumberKeyword(pos, ast));
}

function walkTSNeverKeyword(pos, ast, visitor) {
  const visit = visitor.TSNeverKeyword;
  if (visit !== null) visit(new TSNeverKeyword(pos, ast));
}

function walkTSIntrinsicKeyword(pos, ast, visitor) {
  const visit = visitor.TSIntrinsicKeyword;
  if (visit !== null) visit(new TSIntrinsicKeyword(pos, ast));
}

function walkTSUnknownKeyword(pos, ast, visitor) {
  const visit = visitor.TSUnknownKeyword;
  if (visit !== null) visit(new TSUnknownKeyword(pos, ast));
}

function walkTSNullKeyword(pos, ast, visitor) {
  const visit = visitor.TSNullKeyword;
  if (visit !== null) visit(new TSNullKeyword(pos, ast));
}

function walkTSUndefinedKeyword(pos, ast, visitor) {
  const visit = visitor.TSUndefinedKeyword;
  if (visit !== null) visit(new TSUndefinedKeyword(pos, ast));
}

function walkTSVoidKeyword(pos, ast, visitor) {
  const visit = visitor.TSVoidKeyword;
  if (visit !== null) visit(new TSVoidKeyword(pos, ast));
}

function walkTSSymbolKeyword(pos, ast, visitor) {
  const visit = visitor.TSSymbolKeyword;
  if (visit !== null) visit(new TSSymbolKeyword(pos, ast));
}

function walkTSThisType(pos, ast, visitor) {
  const visit = visitor.TSThisType;
  if (visit !== null) visit(new TSThisType(pos, ast));
}

function walkTSObjectKeyword(pos, ast, visitor) {
  const visit = visitor.TSObjectKeyword;
  if (visit !== null) visit(new TSObjectKeyword(pos, ast));
}

function walkTSBigIntKeyword(pos, ast, visitor) {
  const visit = visitor.TSBigIntKeyword;
  if (visit !== null) visit(new TSBigIntKeyword(pos, ast));
}

function walkTSTypeReference(pos, ast, visitor) {
  const enterExit = visitor.TSTypeReference;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeReference(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSQualifiedName;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSQualifiedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitor);
  walkIdentifierName(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeParameterInstantiation(pos, ast, visitor) {
  const enterExit = visitor.TSTypeParameterInstantiation;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameterInstantiation(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeParameter(pos, ast, visitor) {
  const enterExit = visitor.TSTypeParameter;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameter(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkOptionTSType(pos + 40, ast, visitor);
  walkOptionTSType(pos + 56, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeParameterDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSTypeParameterDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameterDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSTypeParameter(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeAliasDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSTypeAliasDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAliasDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitor);
  walkTSType(pos + 48, ast, visitor);

  if (exit) exit(node);
}

function walkTSClassImplements(pos, ast, visitor) {
  const enterExit = visitor.TSClassImplements;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSClassImplements(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSInterfaceDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSInterfaceDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitor);
  walkVecTSInterfaceHeritage(pos + 48, ast, visitor);
  walkBoxTSInterfaceBody(pos + 72, ast, visitor);

  if (exit) exit(node);
}

function walkTSInterfaceBody(pos, ast, visitor) {
  const enterExit = visitor.TSInterfaceBody;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSPropertySignature(pos, ast, visitor) {
  const enterExit = visitor.TSPropertySignature;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSPropertySignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSIndexSignature;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexSignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSIndexSignatureName(pos + 8, ast, visitor);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSCallSignatureDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSCallSignatureDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSCallSignatureDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 24, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSMethodSignature(pos, ast, visitor) {
  const enterExit = visitor.TSMethodSignature;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSMethodSignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 24, ast, visitor);
  walkBoxFormalParameters(pos + 40, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitor);

  if (exit) exit(node);
}

function walkTSConstructSignatureDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSConstructSignatureDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConstructSignatureDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 16, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSIndexSignatureName(pos, ast, visitor) {
  const enterExit = visitor.TSIndexSignatureName;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexSignatureName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSInterfaceHeritage(pos, ast, visitor) {
  const enterExit = visitor.TSInterfaceHeritage;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceHeritage(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypePredicate(pos, ast, visitor) {
  const enterExit = visitor.TSTypePredicate;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypePredicate(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypePredicateName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSModuleDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSModuleDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSModuleDeclarationName(pos + 8, ast, visitor);
  walkOptionTSModuleDeclarationBody(pos + 64, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSModuleBlock;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSModuleBlock(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeLiteral(pos, ast, visitor) {
  const enterExit = visitor.TSTypeLiteral;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeLiteral(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSInferType(pos, ast, visitor) {
  const enterExit = visitor.TSInferType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInferType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeParameter(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeQuery(pos, ast, visitor) {
  const enterExit = visitor.TSTypeQuery;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeQuery(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeQueryExprName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSImportType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkOptionBoxObjectExpression(pos + 24, ast, visitor);
  walkOptionTSTypeName(pos + 32, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 48, ast, visitor);

  if (exit) exit(node);
}

function walkTSFunctionType(pos, ast, visitor) {
  const enterExit = visitor.TSFunctionType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSFunctionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 24, ast, visitor);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSConstructorType(pos, ast, visitor) {
  const enterExit = visitor.TSConstructorType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConstructorType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 16, ast, visitor);
  walkBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSMappedType(pos, ast, visitor) {
  const enterExit = visitor.TSMappedType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSMappedType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionTSType(pos + 16, ast, visitor);
  walkOptionTSType(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSTemplateLiteralType(pos, ast, visitor) {
  const enterExit = visitor.TSTemplateLiteralType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTemplateLiteralType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 8, ast, visitor);
  walkVecTSType(pos + 32, ast, visitor);

  if (exit) exit(node);
}

function walkTSAsExpression(pos, ast, visitor) {
  const enterExit = visitor.TSAsExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSAsExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSSatisfiesExpression(pos, ast, visitor) {
  const enterExit = visitor.TSSatisfiesExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSSatisfiesExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSTypeAssertion(pos, ast, visitor) {
  const enterExit = visitor.TSTypeAssertion;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAssertion(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkTSImportEqualsDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSImportEqualsDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportEqualsDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkTSModuleReference(pos + 40, ast, visitor);

  if (exit) exit(node);
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
  const enterExit = visitor.TSExternalModuleReference;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExternalModuleReference(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSNonNullExpression(pos, ast, visitor) {
  const enterExit = visitor.TSNonNullExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNonNullExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkDecorator(pos, ast, visitor) {
  const enterExit = visitor.Decorator;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Decorator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSExportAssignment(pos, ast, visitor) {
  const enterExit = visitor.TSExportAssignment;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExportAssignment(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSNamespaceExportDeclaration(pos, ast, visitor) {
  const enterExit = visitor.TSNamespaceExportDeclaration;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamespaceExportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkTSInstantiationExpression(pos, ast, visitor) {
  const enterExit = visitor.TSInstantiationExpression;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInstantiationExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit) exit(node);
}

function walkJSDocNullableType(pos, ast, visitor) {
  const enterExit = visitor.JSDocNullableType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSDocNullableType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkJSDocNonNullableType(pos, ast, visitor) {
  const enterExit = visitor.JSDocNonNullableType;
  let node, enter, exit;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSDocNonNullableType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit) exit(node);
}

function walkJSDocUnknownType(pos, ast, visitor) {
  const visit = visitor.JSDocUnknownType;
  if (visit !== null) visit(new JSDocUnknownType(pos, ast));
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
