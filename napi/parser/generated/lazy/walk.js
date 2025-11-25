// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer_lazy.rs`.

import {
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
  TSGlobalDeclaration,
  TSModuleBlock,
  TSTypeLiteral,
  TSInferType,
  TSTypeQuery,
  TSImportType,
  TSImportTypeQualifiedName,
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
} from "./constructors.js";

export { walkProgram };

function walkProgram(pos, ast, visitors) {
  const enterExit = visitors[38];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Program(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionHashbang(pos + 48, ast, visitors);
  walkVecStatement(pos + 96, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExpression(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Expression`);
  }
}

function walkIdentifierName(pos, ast, visitors) {
  const visit = visitors[0];
  if (visit !== null) visit(new IdentifierName(pos, ast));
}

function walkIdentifierReference(pos, ast, visitors) {
  const visit = visitors[1];
  if (visit !== null) visit(new IdentifierReference(pos, ast));
}

function walkBindingIdentifier(pos, ast, visitors) {
  const visit = visitors[2];
  if (visit !== null) visit(new BindingIdentifier(pos, ast));
}

function walkLabelIdentifier(pos, ast, visitors) {
  const visit = visitors[3];
  if (visit !== null) visit(new LabelIdentifier(pos, ast));
}

function walkThisExpression(pos, ast, visitors) {
  const visit = visitors[4];
  if (visit !== null) visit(new ThisExpression(pos, ast));
}

function walkArrayExpression(pos, ast, visitors) {
  const enterExit = visitors[39];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecArrayExpressionElement(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkArrayExpressionElement(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxSpreadElement(pos + 8, ast, visitors);
      return;
    case 65:
      walkElision(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ArrayExpressionElement`);
  }
}

function walkElision(pos, ast, visitors) {
  const visit = visitors[5];
  if (visit !== null) visit(new Elision(pos, ast));
}

function walkObjectExpression(pos, ast, visitors) {
  const enterExit = visitors[40];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecObjectPropertyKind(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkObjectPropertyKind(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxObjectProperty(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxSpreadElement(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ObjectPropertyKind`);
  }
}

function walkObjectProperty(pos, ast, visitors) {
  const enterExit = visitors[41];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPropertyKey(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxIdentifierName(pos + 8, ast, visitors);
      return;
    case 65:
      walkBoxPrivateIdentifier(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyKey`);
  }
}

function walkTemplateLiteral(pos, ast, visitors) {
  const enterExit = visitors[42];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TemplateLiteral(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 8, ast, visitors);
  walkVecExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTaggedTemplateExpression(pos, ast, visitors) {
  const enterExit = visitors[43];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TaggedTemplateExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);
  walkTemplateLiteral(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTemplateElement(pos, ast, visitors) {
  const visit = visitors[6];
  if (visit !== null) visit(new TemplateElement(pos, ast));
}

function walkComputedMemberExpression(pos, ast, visitors) {
  const enterExit = visitors[44];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ComputedMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkStaticMemberExpression(pos, ast, visitors) {
  const enterExit = visitors[45];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new StaticMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkIdentifierName(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPrivateFieldExpression(pos, ast, visitors) {
  const enterExit = visitors[46];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PrivateFieldExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkPrivateIdentifier(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkCallExpression(pos, ast, visitors) {
  const enterExit = visitors[47];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new CallExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);
  walkVecArgument(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkNewExpression(pos, ast, visitors) {
  const enterExit = visitors[48];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new NewExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);
  walkVecArgument(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkMetaProperty(pos, ast, visitors) {
  const enterExit = visitors[49];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new MetaProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitors);
  walkIdentifierName(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSpreadElement(pos, ast, visitors) {
  const enterExit = visitors[50];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SpreadElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkArgument(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxSpreadElement(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Argument`);
  }
}

function walkUpdateExpression(pos, ast, visitors) {
  const enterExit = visitors[51];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new UpdateExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkSimpleAssignmentTarget(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkUnaryExpression(pos, ast, visitors) {
  const enterExit = visitors[52];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new UnaryExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBinaryExpression(pos, ast, visitors) {
  const enterExit = visitors[53];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BinaryExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPrivateInExpression(pos, ast, visitors) {
  const enterExit = visitors[54];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PrivateInExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPrivateIdentifier(pos + 8, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkLogicalExpression(pos, ast, visitors) {
  const enterExit = visitors[55];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new LogicalExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkConditionalExpression(pos, ast, visitors) {
  const enterExit = visitors[56];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ConditionalExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);
  walkExpression(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentExpression(pos, ast, visitors) {
  const enterExit = visitors[57];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentTarget(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxArrayAssignmentTarget(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxObjectAssignmentTarget(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTarget`);
  }
}

function walkSimpleAssignmentTarget(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for SimpleAssignmentTarget`);
  }
}

function walkArrayAssignmentTarget(pos, ast, visitors) {
  const enterExit = visitors[58];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayAssignmentTarget(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecOptionAssignmentTargetMaybeDefault(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkObjectAssignmentTarget(pos, ast, visitors) {
  const enterExit = visitors[59];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectAssignmentTarget(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecAssignmentTargetProperty(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetMaybeDefault(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxArrayAssignmentTarget(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxObjectAssignmentTarget(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxAssignmentTargetWithDefault(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(
        `Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetMaybeDefault`,
      );
  }
}

function walkAssignmentTargetWithDefault(pos, ast, visitors) {
  const enterExit = visitors[60];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetWithDefault(pos, ast);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetProperty(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxAssignmentTargetPropertyIdentifier(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxAssignmentTargetPropertyProperty(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetProperty`);
  }
}

function walkAssignmentTargetPropertyIdentifier(pos, ast, visitors) {
  const enterExit = visitors[61];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetPropertyIdentifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierReference(pos + 8, ast, visitors);
  walkOptionExpression(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetPropertyProperty(pos, ast, visitors) {
  const enterExit = visitors[62];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetPropertyProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitors);
  walkAssignmentTargetMaybeDefault(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSequenceExpression(pos, ast, visitors) {
  const enterExit = visitors[63];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SequenceExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSuper(pos, ast, visitors) {
  const visit = visitors[7];
  if (visit !== null) visit(new Super(pos, ast));
}

function walkAwaitExpression(pos, ast, visitors) {
  const enterExit = visitors[64];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AwaitExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkChainExpression(pos, ast, visitors) {
  const enterExit = visitors[65];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ChainExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkChainElement(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkChainElement(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ChainElement`);
  }
}

function walkParenthesizedExpression(pos, ast, visitors) {
  const enterExit = visitors[66];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ParenthesizedExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkStatement(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBlockStatement(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxBreakStatement(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxContinueStatement(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxDebuggerStatement(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxDoWhileStatement(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxEmptyStatement(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxExpressionStatement(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxForInStatement(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxForOfStatement(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxForStatement(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxIfStatement(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxLabeledStatement(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxReturnStatement(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxSwitchStatement(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxThrowStatement(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxTryStatement(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxWhileStatement(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxWithStatement(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxVariableDeclaration(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSTypeAliasDeclaration(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSInterfaceDeclaration(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSEnumDeclaration(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSModuleDeclaration(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSGlobalDeclaration(pos + 8, ast, visitors);
      return;
    case 40:
      walkBoxTSImportEqualsDeclaration(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxImportDeclaration(pos + 8, ast, visitors);
      return;
    case 65:
      walkBoxExportAllDeclaration(pos + 8, ast, visitors);
      return;
    case 66:
      walkBoxExportDefaultDeclaration(pos + 8, ast, visitors);
      return;
    case 67:
      walkBoxExportNamedDeclaration(pos + 8, ast, visitors);
      return;
    case 68:
      walkBoxTSExportAssignment(pos + 8, ast, visitors);
      return;
    case 69:
      walkBoxTSNamespaceExportDeclaration(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Statement`);
  }
}

function walkHashbang(pos, ast, visitors) {
  const visit = visitors[8];
  if (visit !== null) visit(new Hashbang(pos, ast));
}

function walkBlockStatement(pos, ast, visitors) {
  const enterExit = visitors[67];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BlockStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkDeclaration(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 32:
      walkBoxVariableDeclaration(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSTypeAliasDeclaration(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSInterfaceDeclaration(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSEnumDeclaration(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSModuleDeclaration(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSGlobalDeclaration(pos + 8, ast, visitors);
      return;
    case 40:
      walkBoxTSImportEqualsDeclaration(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Declaration`);
  }
}

function walkVariableDeclaration(pos, ast, visitors) {
  const enterExit = visitors[68];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new VariableDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecVariableDeclarator(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkVariableDeclarator(pos, ast, visitors) {
  const enterExit = visitors[69];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new VariableDeclarator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 8, ast, visitors);
  walkOptionExpression(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkEmptyStatement(pos, ast, visitors) {
  const visit = visitors[9];
  if (visit !== null) visit(new EmptyStatement(pos, ast));
}

function walkExpressionStatement(pos, ast, visitors) {
  const enterExit = visitors[70];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExpressionStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkIfStatement(pos, ast, visitors) {
  const enterExit = visitors[71];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new IfStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkStatement(pos + 24, ast, visitors);
  walkOptionStatement(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkDoWhileStatement(pos, ast, visitors) {
  const enterExit = visitors[72];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new DoWhileStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStatement(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkWhileStatement(pos, ast, visitors) {
  const enterExit = visitors[73];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new WhileStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkStatement(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkForStatement(pos, ast, visitors) {
  const enterExit = visitors[74];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionForStatementInit(pos + 8, ast, visitors);
  walkOptionExpression(pos + 24, ast, visitors);
  walkOptionExpression(pos + 40, ast, visitors);
  walkStatement(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkForStatementInit(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxVariableDeclaration(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ForStatementInit`);
  }
}

function walkForInStatement(pos, ast, visitors) {
  const enterExit = visitors[75];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForInStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);
  walkStatement(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkForStatementLeft(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxArrayAssignmentTarget(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxObjectAssignmentTarget(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxVariableDeclaration(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ForStatementLeft`);
  }
}

function walkForOfStatement(pos, ast, visitors) {
  const enterExit = visitors[76];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForOfStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);
  walkStatement(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkContinueStatement(pos, ast, visitors) {
  const enterExit = visitors[77];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ContinueStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBreakStatement(pos, ast, visitors) {
  const enterExit = visitors[78];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BreakStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkReturnStatement(pos, ast, visitors) {
  const enterExit = visitors[79];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ReturnStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkWithStatement(pos, ast, visitors) {
  const enterExit = visitors[80];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new WithStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkStatement(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSwitchStatement(pos, ast, visitors) {
  const enterExit = visitors[81];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SwitchStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkVecSwitchCase(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSwitchCase(pos, ast, visitors) {
  const enterExit = visitors[82];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SwitchCase(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitors);
  walkVecStatement(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkLabeledStatement(pos, ast, visitors) {
  const enterExit = visitors[83];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new LabeledStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkLabelIdentifier(pos + 8, ast, visitors);
  walkStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkThrowStatement(pos, ast, visitors) {
  const enterExit = visitors[84];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ThrowStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTryStatement(pos, ast, visitors) {
  const enterExit = visitors[85];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TryStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxBlockStatement(pos + 8, ast, visitors);
  walkOptionBoxCatchClause(pos + 16, ast, visitors);
  walkOptionBoxBlockStatement(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkCatchClause(pos, ast, visitors) {
  const enterExit = visitors[86];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new CatchClause(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionCatchParameter(pos + 8, ast, visitors);
  walkBoxBlockStatement(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkCatchParameter(pos, ast, visitors) {
  walkBindingPattern(pos + 8, ast, visitors);
}

function walkDebuggerStatement(pos, ast, visitors) {
  const visit = visitors[10];
  if (visit !== null) visit(new DebuggerStatement(pos, ast));
}

function walkBindingPattern(pos, ast, visitors) {
  walkBindingPatternKind(pos, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 16, ast, visitors);
}

function walkBindingPatternKind(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBindingIdentifier(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxObjectPattern(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxArrayPattern(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxAssignmentPattern(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for BindingPatternKind`);
  }
}

function walkAssignmentPattern(pos, ast, visitors) {
  const enterExit = visitors[87];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 8, ast, visitors);
  walkExpression(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkObjectPattern(pos, ast, visitors) {
  const enterExit = visitors[88];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecBindingProperty(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBindingProperty(pos, ast, visitors) {
  const enterExit = visitors[89];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BindingProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitors);
  walkBindingPattern(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkArrayPattern(pos, ast, visitors) {
  const enterExit = visitors[90];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecOptionBindingPattern(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkFunction(pos, ast, visitors) {
  const enterExit = visitors[91];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Function(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBindingIdentifier(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitors);
  walkBoxFormalParameters(pos + 56, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 64, ast, visitors);
  walkOptionBoxFunctionBody(pos + 72, ast, visitors);

  if (exit !== null) exit(node);
}

function walkFormalParameters(pos, ast, visitors) {
  const enterExit = visitors[92];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new FormalParameters(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecFormalParameter(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkFormalParameter(pos, ast, visitors) {
  walkVecDecorator(pos + 8, ast, visitors);
  walkBindingPattern(pos + 32, ast, visitors);
}

function walkFunctionBody(pos, ast, visitors) {
  const enterExit = visitors[93];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new FunctionBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkArrowFunctionExpression(pos, ast, visitors) {
  const enterExit = visitors[94];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrowFunctionExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitors);
  walkBoxFormalParameters(pos + 16, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitors);
  walkBoxFunctionBody(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkYieldExpression(pos, ast, visitors) {
  const enterExit = visitors[95];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new YieldExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkClass(pos, ast, visitors) {
  const enterExit = visitors[96];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Class(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitors);
  walkOptionBindingIdentifier(pos + 32, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 64, ast, visitors);
  walkOptionExpression(pos + 72, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 88, ast, visitors);
  walkVecTSClassImplements(pos + 96, ast, visitors);
  walkBoxClassBody(pos + 120, ast, visitors);

  if (exit !== null) exit(node);
}

function walkClassBody(pos, ast, visitors) {
  const enterExit = visitors[97];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ClassBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecClassElement(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkClassElement(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxStaticBlock(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxMethodDefinition(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxPropertyDefinition(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxAccessorProperty(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSIndexSignature(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ClassElement`);
  }
}

function walkMethodDefinition(pos, ast, visitors) {
  const enterExit = visitors[98];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new MethodDefinition(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitors);
  walkPropertyKey(pos + 32, ast, visitors);
  walkBoxFunction(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPropertyDefinition(pos, ast, visitors) {
  const enterExit = visitors[99];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PropertyDefinition(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitors);
  walkPropertyKey(pos + 32, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitors);
  walkOptionExpression(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPrivateIdentifier(pos, ast, visitors) {
  const visit = visitors[11];
  if (visit !== null) visit(new PrivateIdentifier(pos, ast));
}

function walkStaticBlock(pos, ast, visitors) {
  const enterExit = visitors[100];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new StaticBlock(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAccessorProperty(pos, ast, visitors) {
  const enterExit = visitors[101];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AccessorProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitors);
  walkPropertyKey(pos + 32, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitors);
  walkOptionExpression(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportExpression(pos, ast, visitors) {
  const enterExit = visitors[102];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkOptionExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportDeclaration(pos, ast, visitors) {
  const enterExit = visitors[103];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionVecImportDeclarationSpecifier(pos + 8, ast, visitors);
  walkStringLiteral(pos + 32, ast, visitors);
  walkOptionBoxWithClause(pos + 80, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportDeclarationSpecifier(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxImportSpecifier(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxImportDefaultSpecifier(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxImportNamespaceSpecifier(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportDeclarationSpecifier`);
  }
}

function walkImportSpecifier(pos, ast, visitors) {
  const enterExit = visitors[104];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 8, ast, visitors);
  walkBindingIdentifier(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportDefaultSpecifier(pos, ast, visitors) {
  const enterExit = visitors[105];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportDefaultSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportNamespaceSpecifier(pos, ast, visitors) {
  const enterExit = visitors[106];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportNamespaceSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkWithClause(pos, ast, visitors) {
  walkVecImportAttribute(pos + 8, ast, visitors);
}

function walkImportAttribute(pos, ast, visitors) {
  const enterExit = visitors[107];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkImportAttributeKey(pos + 8, ast, visitors);
  walkStringLiteral(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportAttributeKey(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkIdentifierName(pos + 8, ast, visitors);
      return;
    case 1:
      walkStringLiteral(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportAttributeKey`);
  }
}

function walkExportNamedDeclaration(pos, ast, visitors) {
  const enterExit = visitors[108];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportNamedDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionDeclaration(pos + 8, ast, visitors);
  walkVecExportSpecifier(pos + 24, ast, visitors);
  walkOptionStringLiteral(pos + 48, ast, visitors);
  walkOptionBoxWithClause(pos + 96, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportDefaultDeclaration(pos, ast, visitors) {
  const enterExit = visitors[109];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportDefaultDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExportDefaultDeclarationKind(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportAllDeclaration(pos, ast, visitors) {
  const enterExit = visitors[110];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportAllDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionModuleExportName(pos + 8, ast, visitors);
  walkStringLiteral(pos + 64, ast, visitors);
  walkOptionBoxWithClause(pos + 112, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportSpecifier(pos, ast, visitors) {
  const enterExit = visitors[111];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 8, ast, visitors);
  walkModuleExportName(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportDefaultDeclarationKind(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 65:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 66:
      walkBoxTSInterfaceDeclaration(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(
        `Unexpected discriminant ${ast.buffer[pos]} for ExportDefaultDeclarationKind`,
      );
  }
}

function walkModuleExportName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkIdentifierName(pos + 8, ast, visitors);
      return;
    case 1:
      walkIdentifierReference(pos + 8, ast, visitors);
      return;
    case 2:
      walkStringLiteral(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ModuleExportName`);
  }
}

function walkV8IntrinsicExpression(pos, ast, visitors) {
  const enterExit = visitors[112];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new V8IntrinsicExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitors);
  walkVecArgument(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBooleanLiteral(pos, ast, visitors) {
  const visit = visitors[12];
  if (visit !== null) visit(new BooleanLiteral(pos, ast));
}

function walkNullLiteral(pos, ast, visitors) {
  const visit = visitors[13];
  if (visit !== null) visit(new NullLiteral(pos, ast));
}

function walkNumericLiteral(pos, ast, visitors) {
  const visit = visitors[14];
  if (visit !== null) visit(new NumericLiteral(pos, ast));
}

function walkStringLiteral(pos, ast, visitors) {
  const visit = visitors[15];
  if (visit !== null) visit(new StringLiteral(pos, ast));
}

function walkBigIntLiteral(pos, ast, visitors) {
  const visit = visitors[16];
  if (visit !== null) visit(new BigIntLiteral(pos, ast));
}

function walkRegExpLiteral(pos, ast, visitors) {
  const visit = visitors[17];
  if (visit !== null) visit(new RegExpLiteral(pos, ast));
}

function walkJSXElement(pos, ast, visitors) {
  const enterExit = visitors[113];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxJSXOpeningElement(pos + 8, ast, visitors);
  walkVecJSXChild(pos + 16, ast, visitors);
  walkOptionBoxJSXClosingElement(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXOpeningElement(pos, ast, visitors) {
  const enterExit = visitors[114];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXOpeningElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);
  walkVecJSXAttributeItem(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXClosingElement(pos, ast, visitors) {
  const enterExit = visitors[115];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXClosingElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXFragment(pos, ast, visitors) {
  const enterExit = visitors[116];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXFragment(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXOpeningFragment(pos + 8, ast, visitors);
  walkVecJSXChild(pos + 16, ast, visitors);
  walkJSXClosingFragment(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXOpeningFragment(pos, ast, visitors) {
  const visit = visitors[18];
  if (visit !== null) visit(new JSXOpeningFragment(pos, ast));
}

function walkJSXClosingFragment(pos, ast, visitors) {
  const visit = visitors[19];
  if (visit !== null) visit(new JSXClosingFragment(pos, ast));
}

function walkJSXElementName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXIdentifier(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxJSXNamespacedName(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxJSXMemberExpression(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXElementName`);
  }
}

function walkJSXNamespacedName(pos, ast, visitors) {
  const enterExit = visitors[117];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXNamespacedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXIdentifier(pos + 8, ast, visitors);
  walkJSXIdentifier(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXMemberExpression(pos, ast, visitors) {
  const enterExit = visitors[118];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXMemberExpressionObject(pos + 8, ast, visitors);
  walkJSXIdentifier(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXMemberExpressionObject(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxJSXMemberExpression(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXMemberExpressionObject`);
  }
}

function walkJSXExpressionContainer(pos, ast, visitors) {
  const enterExit = visitors[119];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXExpressionContainer(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXExpression(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitors);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitors);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitors);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitors);
      return;
    case 64:
      walkJSXEmptyExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXExpression`);
  }
}

function walkJSXEmptyExpression(pos, ast, visitors) {
  const visit = visitors[20];
  if (visit !== null) visit(new JSXEmptyExpression(pos, ast));
}

function walkJSXAttributeItem(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXAttribute(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxJSXSpreadAttribute(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeItem`);
  }
}

function walkJSXAttribute(pos, ast, visitors) {
  const enterExit = visitors[120];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXAttributeName(pos + 8, ast, visitors);
  walkOptionJSXAttributeValue(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXSpreadAttribute(pos, ast, visitors) {
  const enterExit = visitors[121];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXSpreadAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXAttributeName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXIdentifier(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxJSXNamespacedName(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeName`);
  }
}

function walkJSXAttributeValue(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxJSXExpressionContainer(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeValue`);
  }
}

function walkJSXIdentifier(pos, ast, visitors) {
  const visit = visitors[21];
  if (visit !== null) visit(new JSXIdentifier(pos, ast));
}

function walkJSXChild(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXText(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxJSXExpressionContainer(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxJSXSpreadChild(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXChild`);
  }
}

function walkJSXSpreadChild(pos, ast, visitors) {
  const enterExit = visitors[122];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXSpreadChild(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXText(pos, ast, visitors) {
  const visit = visitors[22];
  if (visit !== null) visit(new JSXText(pos, ast));
}

function walkTSEnumDeclaration(pos, ast, visitors) {
  const enterExit = visitors[123];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);
  walkTSEnumBody(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSEnumBody(pos, ast, visitors) {
  const enterExit = visitors[124];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSEnumMember(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSEnumMember(pos, ast, visitors) {
  const enterExit = visitors[125];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumMember(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSEnumMemberName(pos + 8, ast, visitors);
  walkOptionExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSEnumMemberName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierName(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSEnumMemberName`);
  }
}

function walkTSTypeAnnotation(pos, ast, visitors) {
  const enterExit = visitors[126];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAnnotation(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSLiteralType(pos, ast, visitors) {
  const enterExit = visitors[127];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSLiteralType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSLiteral(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSLiteral(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxNumericLiteral(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxBigIntLiteral(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxStringLiteral(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTemplateLiteral(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSLiteral`);
  }
}

function walkTSType(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSAnyKeyword(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSBigIntKeyword(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSBooleanKeyword(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSIntrinsicKeyword(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSNeverKeyword(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxTSNullKeyword(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTSNumberKeyword(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxTSObjectKeyword(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxTSStringKeyword(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxTSSymbolKeyword(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxTSThisType(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxTSUndefinedKeyword(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxTSUnknownKeyword(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxTSVoidKeyword(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxTSArrayType(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxTSConditionalType(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxTSConstructorType(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxTSFunctionType(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxTSImportType(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxTSIndexedAccessType(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxTSInferType(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxTSIntersectionType(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxTSLiteralType(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxTSMappedType(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxTSNamedTupleMember(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTSTemplateLiteralType(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxTSTupleType(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxTSTypeLiteral(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxTSTypeOperator(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxTSTypePredicate(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxTSTypeQuery(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxTSTypeReference(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxTSUnionType(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSParenthesizedType(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxJSDocNullableType(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxJSDocNonNullableType(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxJSDocUnknownType(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSType`);
  }
}

function walkTSConditionalType(pos, ast, visitors) {
  const enterExit = visitors[128];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConditionalType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);
  walkTSType(pos + 24, ast, visitors);
  walkTSType(pos + 40, ast, visitors);
  walkTSType(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSUnionType(pos, ast, visitors) {
  const enterExit = visitors[129];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSUnionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSIntersectionType(pos, ast, visitors) {
  const enterExit = visitors[130];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIntersectionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSParenthesizedType(pos, ast, visitors) {
  const enterExit = visitors[131];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSParenthesizedType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeOperator(pos, ast, visitors) {
  const enterExit = visitors[132];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeOperator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSArrayType(pos, ast, visitors) {
  const enterExit = visitors[133];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSArrayType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSIndexedAccessType(pos, ast, visitors) {
  const enterExit = visitors[134];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexedAccessType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);
  walkTSType(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTupleType(pos, ast, visitors) {
  const enterExit = visitors[135];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTupleType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSTupleElement(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNamedTupleMember(pos, ast, visitors) {
  const enterExit = visitors[136];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamedTupleMember(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitors);
  walkTSTupleElement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSOptionalType(pos, ast, visitors) {
  const enterExit = visitors[137];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSOptionalType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSRestType(pos, ast, visitors) {
  const enterExit = visitors[138];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSRestType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTupleElement(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSAnyKeyword(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSBigIntKeyword(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSBooleanKeyword(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSIntrinsicKeyword(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSNeverKeyword(pos + 8, ast, visitors);
      return;
    case 5:
      walkBoxTSNullKeyword(pos + 8, ast, visitors);
      return;
    case 6:
      walkBoxTSNumberKeyword(pos + 8, ast, visitors);
      return;
    case 7:
      walkBoxTSObjectKeyword(pos + 8, ast, visitors);
      return;
    case 8:
      walkBoxTSStringKeyword(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxTSSymbolKeyword(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxTSThisType(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxTSUndefinedKeyword(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxTSUnknownKeyword(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxTSVoidKeyword(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxTSArrayType(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxTSConditionalType(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxTSConstructorType(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxTSFunctionType(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxTSImportType(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxTSIndexedAccessType(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxTSInferType(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxTSIntersectionType(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxTSLiteralType(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxTSMappedType(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxTSNamedTupleMember(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxTSTemplateLiteralType(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxTSTupleType(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxTSTypeLiteral(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxTSTypeOperator(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxTSTypePredicate(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxTSTypeQuery(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxTSTypeReference(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxTSUnionType(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxTSParenthesizedType(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxJSDocNullableType(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxJSDocNonNullableType(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxJSDocUnknownType(pos + 8, ast, visitors);
      return;
    case 64:
      walkBoxTSOptionalType(pos + 8, ast, visitors);
      return;
    case 65:
      walkBoxTSRestType(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTupleElement`);
  }
}

function walkTSAnyKeyword(pos, ast, visitors) {
  const visit = visitors[23];
  if (visit !== null) visit(new TSAnyKeyword(pos, ast));
}

function walkTSStringKeyword(pos, ast, visitors) {
  const visit = visitors[24];
  if (visit !== null) visit(new TSStringKeyword(pos, ast));
}

function walkTSBooleanKeyword(pos, ast, visitors) {
  const visit = visitors[25];
  if (visit !== null) visit(new TSBooleanKeyword(pos, ast));
}

function walkTSNumberKeyword(pos, ast, visitors) {
  const visit = visitors[26];
  if (visit !== null) visit(new TSNumberKeyword(pos, ast));
}

function walkTSNeverKeyword(pos, ast, visitors) {
  const visit = visitors[27];
  if (visit !== null) visit(new TSNeverKeyword(pos, ast));
}

function walkTSIntrinsicKeyword(pos, ast, visitors) {
  const visit = visitors[28];
  if (visit !== null) visit(new TSIntrinsicKeyword(pos, ast));
}

function walkTSUnknownKeyword(pos, ast, visitors) {
  const visit = visitors[29];
  if (visit !== null) visit(new TSUnknownKeyword(pos, ast));
}

function walkTSNullKeyword(pos, ast, visitors) {
  const visit = visitors[30];
  if (visit !== null) visit(new TSNullKeyword(pos, ast));
}

function walkTSUndefinedKeyword(pos, ast, visitors) {
  const visit = visitors[31];
  if (visit !== null) visit(new TSUndefinedKeyword(pos, ast));
}

function walkTSVoidKeyword(pos, ast, visitors) {
  const visit = visitors[32];
  if (visit !== null) visit(new TSVoidKeyword(pos, ast));
}

function walkTSSymbolKeyword(pos, ast, visitors) {
  const visit = visitors[33];
  if (visit !== null) visit(new TSSymbolKeyword(pos, ast));
}

function walkTSThisType(pos, ast, visitors) {
  const visit = visitors[34];
  if (visit !== null) visit(new TSThisType(pos, ast));
}

function walkTSObjectKeyword(pos, ast, visitors) {
  const visit = visitors[35];
  if (visit !== null) visit(new TSObjectKeyword(pos, ast));
}

function walkTSBigIntKeyword(pos, ast, visitors) {
  const visit = visitors[36];
  if (visit !== null) visit(new TSBigIntKeyword(pos, ast));
}

function walkTSTypeReference(pos, ast, visitors) {
  const enterExit = visitors[139];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeReference(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSQualifiedName(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeName`);
  }
}

function walkTSQualifiedName(pos, ast, visitors) {
  const enterExit = visitors[140];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSQualifiedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitors);
  walkIdentifierName(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeParameterInstantiation(pos, ast, visitors) {
  const enterExit = visitors[141];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameterInstantiation(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeParameter(pos, ast, visitors) {
  const enterExit = visitors[142];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameter(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);
  walkOptionTSType(pos + 40, ast, visitors);
  walkOptionTSType(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeParameterDeclaration(pos, ast, visitors) {
  const enterExit = visitors[143];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameterDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSTypeParameter(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeAliasDeclaration(pos, ast, visitors) {
  const enterExit = visitors[144];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAliasDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitors);
  walkTSType(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSClassImplements(pos, ast, visitors) {
  const enterExit = visitors[145];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSClassImplements(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInterfaceDeclaration(pos, ast, visitors) {
  const enterExit = visitors[146];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitors);
  walkVecTSInterfaceHeritage(pos + 48, ast, visitors);
  walkBoxTSInterfaceBody(pos + 72, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInterfaceBody(pos, ast, visitors) {
  const enterExit = visitors[147];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSPropertySignature(pos, ast, visitors) {
  const enterExit = visitors[148];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSPropertySignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSSignature(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSIndexSignature(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSPropertySignature(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSCallSignatureDeclaration(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSConstructSignatureDeclaration(pos + 8, ast, visitors);
      return;
    case 4:
      walkBoxTSMethodSignature(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSSignature`);
  }
}

function walkTSIndexSignature(pos, ast, visitors) {
  const enterExit = visitors[149];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexSignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSIndexSignatureName(pos + 8, ast, visitors);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSCallSignatureDeclaration(pos, ast, visitors) {
  const enterExit = visitors[150];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSCallSignatureDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitors);
  walkBoxFormalParameters(pos + 24, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSMethodSignature(pos, ast, visitors) {
  const enterExit = visitors[151];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSMethodSignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 24, ast, visitors);
  walkBoxFormalParameters(pos + 40, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSConstructSignatureDeclaration(pos, ast, visitors) {
  const enterExit = visitors[152];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConstructSignatureDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitors);
  walkBoxFormalParameters(pos + 16, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSIndexSignatureName(pos, ast, visitors) {
  const enterExit = visitors[153];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexSignatureName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeAnnotation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInterfaceHeritage(pos, ast, visitors) {
  const enterExit = visitors[154];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceHeritage(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypePredicate(pos, ast, visitors) {
  const enterExit = visitors[155];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypePredicate(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypePredicateName(pos + 8, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypePredicateName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierName(pos + 8, ast, visitors);
      return;
    case 1:
      walkTSThisType(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypePredicateName`);
  }
}

function walkTSModuleDeclaration(pos, ast, visitors) {
  const enterExit = visitors[156];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSModuleDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSModuleDeclarationName(pos + 8, ast, visitors);
  walkOptionTSModuleDeclarationBody(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSModuleDeclarationName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBindingIdentifier(pos + 8, ast, visitors);
      return;
    case 1:
      walkStringLiteral(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationName`);
  }
}

function walkTSModuleDeclarationBody(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSModuleDeclaration(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSModuleBlock(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationBody`);
  }
}

function walkTSGlobalDeclaration(pos, ast, visitors) {
  const enterExit = visitors[157];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSGlobalDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSModuleBlock(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSModuleBlock(pos, ast, visitors) {
  const enterExit = visitors[158];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSModuleBlock(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeLiteral(pos, ast, visitors) {
  const enterExit = visitors[159];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeLiteral(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInferType(pos, ast, visitors) {
  const enterExit = visitors[160];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInferType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeParameter(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeQuery(pos, ast, visitors) {
  const enterExit = visitors[161];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeQuery(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeQueryExprName(pos + 8, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeQueryExprName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSQualifiedName(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSImportType(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeQueryExprName`);
  }
}

function walkTSImportType(pos, ast, visitors) {
  const enterExit = visitors[162];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 8, ast, visitors);
  walkOptionBoxObjectExpression(pos + 56, ast, visitors);
  walkOptionTSImportTypeQualifier(pos + 64, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 80, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSImportTypeQualifier(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierName(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSImportTypeQualifiedName(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSImportTypeQualifier`);
  }
}

function walkTSImportTypeQualifiedName(pos, ast, visitors) {
  const enterExit = visitors[163];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportTypeQualifiedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSImportTypeQualifier(pos + 8, ast, visitors);
  walkIdentifierName(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSFunctionType(pos, ast, visitors) {
  const enterExit = visitors[164];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSFunctionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitors);
  walkBoxFormalParameters(pos + 24, ast, visitors);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSConstructorType(pos, ast, visitors) {
  const enterExit = visitors[165];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConstructorType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitors);
  walkBoxFormalParameters(pos + 16, ast, visitors);
  walkBoxTSTypeAnnotation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSMappedType(pos, ast, visitors) {
  const enterExit = visitors[166];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSMappedType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionTSType(pos + 16, ast, visitors);
  walkOptionTSType(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTemplateLiteralType(pos, ast, visitors) {
  const enterExit = visitors[167];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTemplateLiteralType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 8, ast, visitors);
  walkVecTSType(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSAsExpression(pos, ast, visitors) {
  const enterExit = visitors[168];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSAsExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkTSType(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSSatisfiesExpression(pos, ast, visitors) {
  const enterExit = visitors[169];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSSatisfiesExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkTSType(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeAssertion(pos, ast, visitors) {
  const enterExit = visitors[170];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAssertion(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);
  walkExpression(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSImportEqualsDeclaration(pos, ast, visitors) {
  const enterExit = visitors[171];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportEqualsDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitors);
  walkTSModuleReference(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSModuleReference(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSQualifiedName(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 3:
      walkBoxTSExternalModuleReference(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleReference`);
  }
}

function walkTSExternalModuleReference(pos, ast, visitors) {
  const enterExit = visitors[172];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExternalModuleReference(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNonNullExpression(pos, ast, visitors) {
  const enterExit = visitors[173];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNonNullExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkDecorator(pos, ast, visitors) {
  const enterExit = visitors[174];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Decorator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSExportAssignment(pos, ast, visitors) {
  const enterExit = visitors[175];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExportAssignment(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNamespaceExportDeclaration(pos, ast, visitors) {
  const enterExit = visitors[176];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamespaceExportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInstantiationExpression(pos, ast, visitors) {
  const enterExit = visitors[177];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInstantiationExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitors);
  walkBoxTSTypeParameterInstantiation(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSDocNullableType(pos, ast, visitors) {
  const enterExit = visitors[178];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSDocNullableType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSDocNonNullableType(pos, ast, visitors) {
  const enterExit = visitors[179];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSDocNonNullableType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSDocUnknownType(pos, ast, visitors) {
  const visit = visitors[37];
  if (visit !== null) visit(new JSDocUnknownType(pos, ast));
}

function walkOptionHashbang(pos, ast, visitors) {
  if (!(ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0))
    walkHashbang(pos, ast, visitors);
}

function walkVecStatement(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkStatement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxBooleanLiteral(pos, ast, visitors) {
  return walkBooleanLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxNullLiteral(pos, ast, visitors) {
  return walkNullLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxNumericLiteral(pos, ast, visitors) {
  return walkNumericLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxBigIntLiteral(pos, ast, visitors) {
  return walkBigIntLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxRegExpLiteral(pos, ast, visitors) {
  return walkRegExpLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxStringLiteral(pos, ast, visitors) {
  return walkStringLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTemplateLiteral(pos, ast, visitors) {
  return walkTemplateLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxIdentifierReference(pos, ast, visitors) {
  return walkIdentifierReference(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxMetaProperty(pos, ast, visitors) {
  return walkMetaProperty(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxSuper(pos, ast, visitors) {
  return walkSuper(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxArrayExpression(pos, ast, visitors) {
  return walkArrayExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxArrowFunctionExpression(pos, ast, visitors) {
  return walkArrowFunctionExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxAssignmentExpression(pos, ast, visitors) {
  return walkAssignmentExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxAwaitExpression(pos, ast, visitors) {
  return walkAwaitExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxBinaryExpression(pos, ast, visitors) {
  return walkBinaryExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxCallExpression(pos, ast, visitors) {
  return walkCallExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxChainExpression(pos, ast, visitors) {
  return walkChainExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxClass(pos, ast, visitors) {
  return walkClass(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxConditionalExpression(pos, ast, visitors) {
  return walkConditionalExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxFunction(pos, ast, visitors) {
  return walkFunction(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxImportExpression(pos, ast, visitors) {
  return walkImportExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxLogicalExpression(pos, ast, visitors) {
  return walkLogicalExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxNewExpression(pos, ast, visitors) {
  return walkNewExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxObjectExpression(pos, ast, visitors) {
  return walkObjectExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxParenthesizedExpression(pos, ast, visitors) {
  return walkParenthesizedExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxSequenceExpression(pos, ast, visitors) {
  return walkSequenceExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTaggedTemplateExpression(pos, ast, visitors) {
  return walkTaggedTemplateExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxThisExpression(pos, ast, visitors) {
  return walkThisExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxUnaryExpression(pos, ast, visitors) {
  return walkUnaryExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxUpdateExpression(pos, ast, visitors) {
  return walkUpdateExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxYieldExpression(pos, ast, visitors) {
  return walkYieldExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxPrivateInExpression(pos, ast, visitors) {
  return walkPrivateInExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXElement(pos, ast, visitors) {
  return walkJSXElement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXFragment(pos, ast, visitors) {
  return walkJSXFragment(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSAsExpression(pos, ast, visitors) {
  return walkTSAsExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSSatisfiesExpression(pos, ast, visitors) {
  return walkTSSatisfiesExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeAssertion(pos, ast, visitors) {
  return walkTSTypeAssertion(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSNonNullExpression(pos, ast, visitors) {
  return walkTSNonNullExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSInstantiationExpression(pos, ast, visitors) {
  return walkTSInstantiationExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxV8IntrinsicExpression(pos, ast, visitors) {
  return walkV8IntrinsicExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecArrayExpressionElement(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkArrayExpressionElement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxSpreadElement(pos, ast, visitors) {
  return walkSpreadElement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecObjectPropertyKind(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkObjectPropertyKind(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxObjectProperty(pos, ast, visitors) {
  return walkObjectProperty(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxIdentifierName(pos, ast, visitors) {
  return walkIdentifierName(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxPrivateIdentifier(pos, ast, visitors) {
  return walkPrivateIdentifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecTemplateElement(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkTemplateElement(pos, ast, visitors);
    pos += 48;
  }
}

function walkVecExpression(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkExpression(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxTSTypeParameterInstantiation(pos, ast, visitors) {
  return walkTSTypeParameterInstantiation(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxTSTypeParameterInstantiation(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxTSTypeParameterInstantiation(pos, ast, visitors);
}

function walkBoxComputedMemberExpression(pos, ast, visitors) {
  return walkComputedMemberExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxStaticMemberExpression(pos, ast, visitors) {
  return walkStaticMemberExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxPrivateFieldExpression(pos, ast, visitors) {
  return walkPrivateFieldExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecArgument(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkArgument(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxArrayAssignmentTarget(pos, ast, visitors) {
  return walkArrayAssignmentTarget(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxObjectAssignmentTarget(pos, ast, visitors) {
  return walkObjectAssignmentTarget(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionAssignmentTargetMaybeDefault(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 51)) walkAssignmentTargetMaybeDefault(pos, ast, visitors);
}

function walkVecOptionAssignmentTargetMaybeDefault(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkOptionAssignmentTargetMaybeDefault(pos, ast, visitors);
    pos += 16;
  }
}

function walkVecAssignmentTargetProperty(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkAssignmentTargetProperty(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxAssignmentTargetWithDefault(pos, ast, visitors) {
  return walkAssignmentTargetWithDefault(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxAssignmentTargetPropertyIdentifier(pos, ast, visitors) {
  return walkAssignmentTargetPropertyIdentifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxAssignmentTargetPropertyProperty(pos, ast, visitors) {
  return walkAssignmentTargetPropertyProperty(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionExpression(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 51)) walkExpression(pos, ast, visitors);
}

function walkBoxBlockStatement(pos, ast, visitors) {
  return walkBlockStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxBreakStatement(pos, ast, visitors) {
  return walkBreakStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxContinueStatement(pos, ast, visitors) {
  return walkContinueStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxDebuggerStatement(pos, ast, visitors) {
  return walkDebuggerStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxDoWhileStatement(pos, ast, visitors) {
  return walkDoWhileStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxEmptyStatement(pos, ast, visitors) {
  return walkEmptyStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxExpressionStatement(pos, ast, visitors) {
  return walkExpressionStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxForInStatement(pos, ast, visitors) {
  return walkForInStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxForOfStatement(pos, ast, visitors) {
  return walkForOfStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxForStatement(pos, ast, visitors) {
  return walkForStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxIfStatement(pos, ast, visitors) {
  return walkIfStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxLabeledStatement(pos, ast, visitors) {
  return walkLabeledStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxReturnStatement(pos, ast, visitors) {
  return walkReturnStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxSwitchStatement(pos, ast, visitors) {
  return walkSwitchStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxThrowStatement(pos, ast, visitors) {
  return walkThrowStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTryStatement(pos, ast, visitors) {
  return walkTryStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxWhileStatement(pos, ast, visitors) {
  return walkWhileStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxWithStatement(pos, ast, visitors) {
  return walkWithStatement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxVariableDeclaration(pos, ast, visitors) {
  return walkVariableDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeAliasDeclaration(pos, ast, visitors) {
  return walkTSTypeAliasDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSInterfaceDeclaration(pos, ast, visitors) {
  return walkTSInterfaceDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSEnumDeclaration(pos, ast, visitors) {
  return walkTSEnumDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSModuleDeclaration(pos, ast, visitors) {
  return walkTSModuleDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSGlobalDeclaration(pos, ast, visitors) {
  return walkTSGlobalDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSImportEqualsDeclaration(pos, ast, visitors) {
  return walkTSImportEqualsDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecVariableDeclarator(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 64;
  while (pos < endPos) {
    walkVariableDeclarator(pos, ast, visitors);
    pos += 64;
  }
}

function walkOptionStatement(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 70)) walkStatement(pos, ast, visitors);
}

function walkOptionForStatementInit(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 65)) walkForStatementInit(pos, ast, visitors);
}

function walkOptionLabelIdentifier(pos, ast, visitors) {
  if (!(ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0))
    walkLabelIdentifier(pos, ast, visitors);
}

function walkVecSwitchCase(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkSwitchCase(pos, ast, visitors);
    pos += 48;
  }
}

function walkBoxCatchClause(pos, ast, visitors) {
  return walkCatchClause(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxCatchClause(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxCatchClause(pos, ast, visitors);
}

function walkOptionBoxBlockStatement(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxBlockStatement(pos, ast, visitors);
}

function walkOptionCatchParameter(pos, ast, visitors) {
  if (!(ast.buffer[pos + 32] === 2)) walkCatchParameter(pos, ast, visitors);
}

function walkBoxTSTypeAnnotation(pos, ast, visitors) {
  return walkTSTypeAnnotation(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxTSTypeAnnotation(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxTSTypeAnnotation(pos, ast, visitors);
}

function walkBoxBindingIdentifier(pos, ast, visitors) {
  return walkBindingIdentifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxObjectPattern(pos, ast, visitors) {
  return walkObjectPattern(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxArrayPattern(pos, ast, visitors) {
  return walkArrayPattern(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxAssignmentPattern(pos, ast, visitors) {
  return walkAssignmentPattern(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecBindingProperty(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 64;
  while (pos < endPos) {
    walkBindingProperty(pos, ast, visitors);
    pos += 64;
  }
}

function walkOptionBindingPattern(pos, ast, visitors) {
  if (!(ast.buffer[pos + 24] === 2)) walkBindingPattern(pos, ast, visitors);
}

function walkVecOptionBindingPattern(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkOptionBindingPattern(pos, ast, visitors);
    pos += 32;
  }
}

function walkOptionBindingIdentifier(pos, ast, visitors) {
  if (!(ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0))
    walkBindingIdentifier(pos, ast, visitors);
}

function walkBoxTSTypeParameterDeclaration(pos, ast, visitors) {
  return walkTSTypeParameterDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxTSTypeParameterDeclaration(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxTSTypeParameterDeclaration(pos, ast, visitors);
}

function walkBoxFormalParameters(pos, ast, visitors) {
  return walkFormalParameters(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxFunctionBody(pos, ast, visitors) {
  return walkFunctionBody(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxFunctionBody(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxFunctionBody(pos, ast, visitors);
}

function walkVecFormalParameter(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 72;
  while (pos < endPos) {
    walkFormalParameter(pos, ast, visitors);
    pos += 72;
  }
}

function walkVecDecorator(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 24;
  while (pos < endPos) {
    walkDecorator(pos, ast, visitors);
    pos += 24;
  }
}

function walkVecTSClassImplements(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkTSClassImplements(pos, ast, visitors);
    pos += 32;
  }
}

function walkBoxClassBody(pos, ast, visitors) {
  return walkClassBody(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecClassElement(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkClassElement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxStaticBlock(pos, ast, visitors) {
  return walkStaticBlock(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxMethodDefinition(pos, ast, visitors) {
  return walkMethodDefinition(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxPropertyDefinition(pos, ast, visitors) {
  return walkPropertyDefinition(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxAccessorProperty(pos, ast, visitors) {
  return walkAccessorProperty(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSIndexSignature(pos, ast, visitors) {
  return walkTSIndexSignature(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxImportDeclaration(pos, ast, visitors) {
  return walkImportDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxExportAllDeclaration(pos, ast, visitors) {
  return walkExportAllDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxExportDefaultDeclaration(pos, ast, visitors) {
  return walkExportDefaultDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxExportNamedDeclaration(pos, ast, visitors) {
  return walkExportNamedDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSExportAssignment(pos, ast, visitors) {
  return walkTSExportAssignment(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSNamespaceExportDeclaration(pos, ast, visitors) {
  return walkTSNamespaceExportDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecImportDeclarationSpecifier(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkImportDeclarationSpecifier(pos, ast, visitors);
    pos += 16;
  }
}

function walkOptionVecImportDeclarationSpecifier(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkVecImportDeclarationSpecifier(pos, ast, visitors);
}

function walkBoxWithClause(pos, ast, visitors) {
  return walkWithClause(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxWithClause(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxWithClause(pos, ast, visitors);
}

function walkBoxImportSpecifier(pos, ast, visitors) {
  return walkImportSpecifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxImportDefaultSpecifier(pos, ast, visitors) {
  return walkImportDefaultSpecifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxImportNamespaceSpecifier(pos, ast, visitors) {
  return walkImportNamespaceSpecifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecImportAttribute(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 112;
  while (pos < endPos) {
    walkImportAttribute(pos, ast, visitors);
    pos += 112;
  }
}

function walkOptionDeclaration(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 31)) walkDeclaration(pos, ast, visitors);
}

function walkVecExportSpecifier(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 128;
  while (pos < endPos) {
    walkExportSpecifier(pos, ast, visitors);
    pos += 128;
  }
}

function walkOptionStringLiteral(pos, ast, visitors) {
  if (!(ast.buffer[pos + 40] === 2)) walkStringLiteral(pos, ast, visitors);
}

function walkOptionModuleExportName(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 3)) walkModuleExportName(pos, ast, visitors);
}

function walkBoxJSXOpeningElement(pos, ast, visitors) {
  return walkJSXOpeningElement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecJSXChild(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkJSXChild(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxJSXClosingElement(pos, ast, visitors) {
  return walkJSXClosingElement(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxJSXClosingElement(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxJSXClosingElement(pos, ast, visitors);
}

function walkVecJSXAttributeItem(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkJSXAttributeItem(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxJSXIdentifier(pos, ast, visitors) {
  return walkJSXIdentifier(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXNamespacedName(pos, ast, visitors) {
  return walkJSXNamespacedName(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXMemberExpression(pos, ast, visitors) {
  return walkJSXMemberExpression(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXAttribute(pos, ast, visitors) {
  return walkJSXAttribute(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXSpreadAttribute(pos, ast, visitors) {
  return walkJSXSpreadAttribute(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionJSXAttributeValue(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 4)) walkJSXAttributeValue(pos, ast, visitors);
}

function walkBoxJSXExpressionContainer(pos, ast, visitors) {
  return walkJSXExpressionContainer(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXText(pos, ast, visitors) {
  return walkJSXText(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSXSpreadChild(pos, ast, visitors) {
  return walkJSXSpreadChild(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecTSEnumMember(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 40;
  while (pos < endPos) {
    walkTSEnumMember(pos, ast, visitors);
    pos += 40;
  }
}

function walkBoxTSAnyKeyword(pos, ast, visitors) {
  return walkTSAnyKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSBigIntKeyword(pos, ast, visitors) {
  return walkTSBigIntKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSBooleanKeyword(pos, ast, visitors) {
  return walkTSBooleanKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSIntrinsicKeyword(pos, ast, visitors) {
  return walkTSIntrinsicKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSNeverKeyword(pos, ast, visitors) {
  return walkTSNeverKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSNullKeyword(pos, ast, visitors) {
  return walkTSNullKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSNumberKeyword(pos, ast, visitors) {
  return walkTSNumberKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSObjectKeyword(pos, ast, visitors) {
  return walkTSObjectKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSStringKeyword(pos, ast, visitors) {
  return walkTSStringKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSSymbolKeyword(pos, ast, visitors) {
  return walkTSSymbolKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSUndefinedKeyword(pos, ast, visitors) {
  return walkTSUndefinedKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSUnknownKeyword(pos, ast, visitors) {
  return walkTSUnknownKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSVoidKeyword(pos, ast, visitors) {
  return walkTSVoidKeyword(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSArrayType(pos, ast, visitors) {
  return walkTSArrayType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSConditionalType(pos, ast, visitors) {
  return walkTSConditionalType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSConstructorType(pos, ast, visitors) {
  return walkTSConstructorType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSFunctionType(pos, ast, visitors) {
  return walkTSFunctionType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSImportType(pos, ast, visitors) {
  return walkTSImportType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSIndexedAccessType(pos, ast, visitors) {
  return walkTSIndexedAccessType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSInferType(pos, ast, visitors) {
  return walkTSInferType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSIntersectionType(pos, ast, visitors) {
  return walkTSIntersectionType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSLiteralType(pos, ast, visitors) {
  return walkTSLiteralType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSMappedType(pos, ast, visitors) {
  return walkTSMappedType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSNamedTupleMember(pos, ast, visitors) {
  return walkTSNamedTupleMember(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTemplateLiteralType(pos, ast, visitors) {
  return walkTSTemplateLiteralType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSThisType(pos, ast, visitors) {
  return walkTSThisType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTupleType(pos, ast, visitors) {
  return walkTSTupleType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeLiteral(pos, ast, visitors) {
  return walkTSTypeLiteral(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeOperator(pos, ast, visitors) {
  return walkTSTypeOperator(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypePredicate(pos, ast, visitors) {
  return walkTSTypePredicate(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeQuery(pos, ast, visitors) {
  return walkTSTypeQuery(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeReference(pos, ast, visitors) {
  return walkTSTypeReference(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSUnionType(pos, ast, visitors) {
  return walkTSUnionType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSParenthesizedType(pos, ast, visitors) {
  return walkTSParenthesizedType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSDocNullableType(pos, ast, visitors) {
  return walkJSDocNullableType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSDocNonNullableType(pos, ast, visitors) {
  return walkJSDocNonNullableType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxJSDocUnknownType(pos, ast, visitors) {
  return walkJSDocUnknownType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecTSType(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSType(pos, ast, visitors);
    pos += 16;
  }
}

function walkVecTSTupleElement(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSTupleElement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxTSOptionalType(pos, ast, visitors) {
  return walkTSOptionalType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSRestType(pos, ast, visitors) {
  return walkTSRestType(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSQualifiedName(pos, ast, visitors) {
  return walkTSQualifiedName(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionTSType(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 38)) walkTSType(pos, ast, visitors);
}

function walkVecTSTypeParameter(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 80;
  while (pos < endPos) {
    walkTSTypeParameter(pos, ast, visitors);
    pos += 80;
  }
}

function walkVecTSInterfaceHeritage(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkTSInterfaceHeritage(pos, ast, visitors);
    pos += 32;
  }
}

function walkBoxTSInterfaceBody(pos, ast, visitors) {
  return walkTSInterfaceBody(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecTSSignature(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSSignature(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxTSPropertySignature(pos, ast, visitors) {
  return walkTSPropertySignature(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSCallSignatureDeclaration(pos, ast, visitors) {
  return walkTSCallSignatureDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSConstructSignatureDeclaration(pos, ast, visitors) {
  return walkTSConstructSignatureDeclaration(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSMethodSignature(pos, ast, visitors) {
  return walkTSMethodSignature(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkVecTSIndexSignatureName(pos, ast, visitors) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkTSIndexSignatureName(pos, ast, visitors);
    pos += 32;
  }
}

function walkOptionTSModuleDeclarationBody(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 2)) walkTSModuleDeclarationBody(pos, ast, visitors);
}

function walkBoxTSModuleBlock(pos, ast, visitors) {
  return walkTSModuleBlock(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSTypeParameter(pos, ast, visitors) {
  return walkTSTypeParameter(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkOptionBoxObjectExpression(pos, ast, visitors) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0))
    walkBoxObjectExpression(pos, ast, visitors);
}

function walkOptionTSImportTypeQualifier(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 2)) walkTSImportTypeQualifier(pos, ast, visitors);
}

function walkBoxTSImportTypeQualifiedName(pos, ast, visitors) {
  return walkTSImportTypeQualifiedName(ast.buffer.uint32[pos >> 2], ast, visitors);
}

function walkBoxTSExternalModuleReference(pos, ast, visitors) {
  return walkTSExternalModuleReference(ast.buffer.uint32[pos >> 2], ast, visitors);
}
