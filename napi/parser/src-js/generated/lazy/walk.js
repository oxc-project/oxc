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
  ImportMeta,
  NewTarget,
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
  ExportDeclaration,
  ExportNamedDeclaration,
  ExportFromDeclaration,
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
  TSExternalModuleDeclaration,
  TSNamespaceDeclaration,
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
  const enterExit = visitors[40];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Program(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionHashbang(pos + 56, ast, visitors);
  walkVecStatement(pos + 112, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
  const enterExit = visitors[41];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecArrayExpressionElement(pos + 16, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
      walkBoxElision(pos + 8, ast, visitors);
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
  const enterExit = visitors[42];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecObjectPropertyKind(pos + 16, ast, visitors);

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
  const enterExit = visitors[43];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
  const enterExit = visitors[44];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TemplateLiteral(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 16, ast, visitors);
  walkVecExpression(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTaggedTemplateExpression(pos, ast, visitors) {
  const enterExit = visitors[45];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TaggedTemplateExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);
  walkTemplateLiteral(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTemplateElement(pos, ast, visitors) {
  const visit = visitors[6];
  if (visit !== null) visit(new TemplateElement(pos, ast));
}

function walkComputedMemberExpression(pos, ast, visitors) {
  const enterExit = visitors[46];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ComputedMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkStaticMemberExpression(pos, ast, visitors) {
  const enterExit = visitors[47];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new StaticMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkIdentifierName(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPrivateFieldExpression(pos, ast, visitors) {
  const enterExit = visitors[48];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PrivateFieldExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkPrivateIdentifier(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkCallExpression(pos, ast, visitors) {
  const enterExit = visitors[49];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new CallExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);
  walkVecArgument(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkNewExpression(pos, ast, visitors) {
  const enterExit = visitors[50];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new NewExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);
  walkVecArgument(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportMeta(pos, ast, visitors) {
  const visit = visitors[7];
  if (visit !== null) visit(new ImportMeta(pos, ast));
}

function walkNewTarget(pos, ast, visitors) {
  const visit = visitors[8];
  if (visit !== null) visit(new NewTarget(pos, ast));
}

function walkSpreadElement(pos, ast, visitors) {
  const enterExit = visitors[51];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SpreadElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
  const enterExit = visitors[52];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new UpdateExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkSimpleAssignmentTarget(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkUnaryExpression(pos, ast, visitors) {
  const enterExit = visitors[53];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new UnaryExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBinaryExpression(pos, ast, visitors) {
  const enterExit = visitors[54];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BinaryExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPrivateInExpression(pos, ast, visitors) {
  const enterExit = visitors[55];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PrivateInExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPrivateIdentifier(pos + 16, ast, visitors);
  walkExpression(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkLogicalExpression(pos, ast, visitors) {
  const enterExit = visitors[56];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new LogicalExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkConditionalExpression(pos, ast, visitors) {
  const enterExit = visitors[57];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ConditionalExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);
  walkExpression(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentExpression(pos, ast, visitors) {
  const enterExit = visitors[58];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

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
  const enterExit = visitors[59];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayAssignmentTarget(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecOptionAssignmentTargetMaybeDefault(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkObjectAssignmentTarget(pos, ast, visitors) {
  const enterExit = visitors[60];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectAssignmentTarget(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecAssignmentTargetProperty(pos + 16, ast, visitors);

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
  const enterExit = visitors[61];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetWithDefault(pos, ast);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

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
  const enterExit = visitors[62];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetPropertyIdentifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierReference(pos + 16, ast, visitors);
  walkOptionExpression(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetPropertyProperty(pos, ast, visitors) {
  const enterExit = visitors[63];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentTargetPropertyProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 16, ast, visitors);
  walkAssignmentTargetMaybeDefault(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSequenceExpression(pos, ast, visitors) {
  const enterExit = visitors[64];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SequenceExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSuper(pos, ast, visitors) {
  const visit = visitors[9];
  if (visit !== null) visit(new Super(pos, ast));
}

function walkAwaitExpression(pos, ast, visitors) {
  const enterExit = visitors[65];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AwaitExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkChainExpression(pos, ast, visitors) {
  const enterExit = visitors[66];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ChainExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkChainElement(pos + 16, ast, visitors);

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
  const enterExit = visitors[67];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ParenthesizedExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

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
      walkBoxTSExternalModuleDeclaration(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSNamespaceDeclaration(pos + 8, ast, visitors);
      return;
    case 40:
      walkBoxTSGlobalDeclaration(pos + 8, ast, visitors);
      return;
    case 41:
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
      walkBoxExportDeclaration(pos + 8, ast, visitors);
      return;
    case 68:
      walkBoxExportNamedDeclaration(pos + 8, ast, visitors);
      return;
    case 69:
      walkBoxExportFromDeclaration(pos + 8, ast, visitors);
      return;
    case 70:
      walkBoxTSExportAssignment(pos + 8, ast, visitors);
      return;
    case 71:
      walkBoxTSNamespaceExportDeclaration(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Statement`);
  }
}

function walkHashbang(pos, ast, visitors) {
  const visit = visitors[10];
  if (visit !== null) visit(new Hashbang(pos, ast));
}

function walkBlockStatement(pos, ast, visitors) {
  const enterExit = visitors[68];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BlockStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 16, ast, visitors);

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
      walkBoxTSExternalModuleDeclaration(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSNamespaceDeclaration(pos + 8, ast, visitors);
      return;
    case 40:
      walkBoxTSGlobalDeclaration(pos + 8, ast, visitors);
      return;
    case 41:
      walkBoxTSImportEqualsDeclaration(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Declaration`);
  }
}

function walkVariableDeclaration(pos, ast, visitors) {
  const enterExit = visitors[69];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new VariableDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecVariableDeclarator(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkVariableDeclarator(pos, ast, visitors) {
  const enterExit = visitors[70];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new VariableDeclarator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 16, ast, visitors);
  walkOptionExpression(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkEmptyStatement(pos, ast, visitors) {
  const visit = visitors[11];
  if (visit !== null) visit(new EmptyStatement(pos, ast));
}

function walkExpressionStatement(pos, ast, visitors) {
  const enterExit = visitors[71];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExpressionStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkIfStatement(pos, ast, visitors) {
  const enterExit = visitors[72];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new IfStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkStatement(pos + 32, ast, visitors);
  walkOptionStatement(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkDoWhileStatement(pos, ast, visitors) {
  const enterExit = visitors[73];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new DoWhileStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStatement(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkWhileStatement(pos, ast, visitors) {
  const enterExit = visitors[74];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new WhileStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkForStatement(pos, ast, visitors) {
  const enterExit = visitors[75];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionForStatementInit(pos + 16, ast, visitors);
  walkOptionExpression(pos + 32, ast, visitors);
  walkOptionExpression(pos + 48, ast, visitors);
  walkStatement(pos + 64, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
  const enterExit = visitors[76];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForInStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);
  walkStatement(pos + 48, ast, visitors);

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
  const enterExit = visitors[77];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ForOfStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);
  walkStatement(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkContinueStatement(pos, ast, visitors) {
  const enterExit = visitors[78];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ContinueStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBreakStatement(pos, ast, visitors) {
  const enterExit = visitors[79];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BreakStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkReturnStatement(pos, ast, visitors) {
  const enterExit = visitors[80];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ReturnStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkWithStatement(pos, ast, visitors) {
  const enterExit = visitors[81];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new WithStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSwitchStatement(pos, ast, visitors) {
  const enterExit = visitors[82];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SwitchStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkVecSwitchCase(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkSwitchCase(pos, ast, visitors) {
  const enterExit = visitors[83];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new SwitchCase(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 16, ast, visitors);
  walkVecStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkLabeledStatement(pos, ast, visitors) {
  const enterExit = visitors[84];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new LabeledStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkLabelIdentifier(pos + 16, ast, visitors);
  walkStatement(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkThrowStatement(pos, ast, visitors) {
  const enterExit = visitors[85];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ThrowStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTryStatement(pos, ast, visitors) {
  const enterExit = visitors[86];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TryStatement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxBlockStatement(pos + 16, ast, visitors);
  walkOptionBoxCatchClause(pos + 24, ast, visitors);
  walkOptionBoxBlockStatement(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkCatchClause(pos, ast, visitors) {
  const enterExit = visitors[87];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new CatchClause(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionCatchParameter(pos + 16, ast, visitors);
  walkBoxBlockStatement(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkCatchParameter(pos, ast, visitors) {
  walkBindingPattern(pos + 16, ast, visitors);
}

function walkDebuggerStatement(pos, ast, visitors) {
  const visit = visitors[12];
  if (visit !== null) visit(new DebuggerStatement(pos, ast));
}

function walkBindingPattern(pos, ast, visitors) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for BindingPattern`);
  }
}

function walkAssignmentPattern(pos, ast, visitors) {
  const enterExit = visitors[88];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AssignmentPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkObjectPattern(pos, ast, visitors) {
  const enterExit = visitors[89];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ObjectPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecBindingProperty(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBindingProperty(pos, ast, visitors) {
  const enterExit = visitors[90];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new BindingProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 16, ast, visitors);
  walkBindingPattern(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkArrayPattern(pos, ast, visitors) {
  const enterExit = visitors[91];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrayPattern(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecOptionBindingPattern(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkFunction(pos, ast, visitors) {
  const enterExit = visitors[92];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Function(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBindingIdentifier(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 48, ast, visitors);
  walkBoxFormalParameters(pos + 64, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 72, ast, visitors);
  walkOptionBoxFunctionBody(pos + 80, ast, visitors);

  if (exit !== null) exit(node);
}

function walkFormalParameters(pos, ast, visitors) {
  const enterExit = visitors[93];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new FormalParameters(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecFormalParameter(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkFormalParameter(pos, ast, visitors) {
  walkVecDecorator(pos + 16, ast, visitors);
  walkBindingPattern(pos + 40, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 56, ast, visitors);
  walkOptionBoxExpression(pos + 64, ast, visitors);
}

function walkFunctionBody(pos, ast, visitors) {
  const enterExit = visitors[94];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new FunctionBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkArrowFunctionBody(pos, ast, visitors) {
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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
      walkBoxFunctionBody(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ArrowFunctionBody`);
  }
}

function walkArrowFunctionExpression(pos, ast, visitors) {
  const enterExit = visitors[95];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ArrowFunctionExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 16, ast, visitors);
  walkBoxFormalParameters(pos + 24, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitors);
  walkArrowFunctionBody(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkYieldExpression(pos, ast, visitors) {
  const enterExit = visitors[96];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new YieldExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkClass(pos, ast, visitors) {
  const enterExit = visitors[97];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Class(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 16, ast, visitors);
  walkOptionBindingIdentifier(pos + 40, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 72, ast, visitors);
  walkVecTSClassImplements(pos + 104, ast, visitors);
  walkBoxClassBody(pos + 128, ast, visitors);

  if (exit !== null) exit(node);
}

function walkClassBody(pos, ast, visitors) {
  const enterExit = visitors[98];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ClassBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecClassElement(pos + 16, ast, visitors);

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
  const enterExit = visitors[99];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new MethodDefinition(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 16, ast, visitors);
  walkPropertyKey(pos + 40, ast, visitors);
  walkBoxFunction(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPropertyDefinition(pos, ast, visitors) {
  const enterExit = visitors[100];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new PropertyDefinition(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 16, ast, visitors);
  walkPropertyKey(pos + 40, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 56, ast, visitors);
  walkOptionExpression(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkPrivateIdentifier(pos, ast, visitors) {
  const visit = visitors[13];
  if (visit !== null) visit(new PrivateIdentifier(pos, ast));
}

function walkStaticBlock(pos, ast, visitors) {
  const enterExit = visitors[101];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new StaticBlock(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkAccessorProperty(pos, ast, visitors) {
  const enterExit = visitors[102];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new AccessorProperty(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 16, ast, visitors);
  walkPropertyKey(pos + 40, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 56, ast, visitors);
  walkOptionExpression(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportExpression(pos, ast, visitors) {
  const enterExit = visitors[103];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkOptionExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportDeclaration(pos, ast, visitors) {
  const enterExit = visitors[104];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionVecImportDeclarationSpecifier(pos + 16, ast, visitors);
  walkStringLiteral(pos + 40, ast, visitors);
  walkOptionBoxWithClause(pos + 88, ast, visitors);

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
  const enterExit = visitors[105];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 16, ast, visitors);
  walkBindingIdentifier(pos + 72, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportDefaultSpecifier(pos, ast, visitors) {
  const enterExit = visitors[106];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportDefaultSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkImportNamespaceSpecifier(pos, ast, visitors) {
  const enterExit = visitors[107];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportNamespaceSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkWithClause(pos, ast, visitors) {
  walkVecImportAttribute(pos + 16, ast, visitors);
}

function walkImportAttribute(pos, ast, visitors) {
  const enterExit = visitors[108];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ImportAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkImportAttributeKey(pos + 16, ast, visitors);
  walkStringLiteral(pos + 72, ast, visitors);

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

function walkExportDeclaration(pos, ast, visitors) {
  const enterExit = visitors[109];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkDeclaration(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportNamedDeclaration(pos, ast, visitors) {
  const enterExit = visitors[110];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportNamedDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecExportSpecifier(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportFromDeclaration(pos, ast, visitors) {
  const enterExit = visitors[111];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportFromDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecExportSpecifier(pos + 16, ast, visitors);
  walkStringLiteral(pos + 40, ast, visitors);
  walkOptionBoxWithClause(pos + 88, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportDefaultDeclaration(pos, ast, visitors) {
  const enterExit = visitors[112];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportDefaultDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExportDefaultDeclarationKind(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportAllDeclaration(pos, ast, visitors) {
  const enterExit = visitors[113];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportAllDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionModuleExportName(pos + 16, ast, visitors);
  walkStringLiteral(pos + 72, ast, visitors);
  walkOptionBoxWithClause(pos + 120, ast, visitors);

  if (exit !== null) exit(node);
}

function walkExportSpecifier(pos, ast, visitors) {
  const enterExit = visitors[114];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new ExportSpecifier(pos, ast);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 16, ast, visitors);
  walkModuleExportName(pos + 72, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
  const enterExit = visitors[115];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new V8IntrinsicExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 16, ast, visitors);
  walkVecArgument(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkBooleanLiteral(pos, ast, visitors) {
  const visit = visitors[14];
  if (visit !== null) visit(new BooleanLiteral(pos, ast));
}

function walkNullLiteral(pos, ast, visitors) {
  const visit = visitors[15];
  if (visit !== null) visit(new NullLiteral(pos, ast));
}

function walkNumericLiteral(pos, ast, visitors) {
  const visit = visitors[16];
  if (visit !== null) visit(new NumericLiteral(pos, ast));
}

function walkStringLiteral(pos, ast, visitors) {
  const visit = visitors[17];
  if (visit !== null) visit(new StringLiteral(pos, ast));
}

function walkBigIntLiteral(pos, ast, visitors) {
  const visit = visitors[18];
  if (visit !== null) visit(new BigIntLiteral(pos, ast));
}

function walkRegExpLiteral(pos, ast, visitors) {
  const visit = visitors[19];
  if (visit !== null) visit(new RegExpLiteral(pos, ast));
}

function walkJSXElement(pos, ast, visitors) {
  const enterExit = visitors[116];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxJSXOpeningElement(pos + 16, ast, visitors);
  walkVecJSXChild(pos + 24, ast, visitors);
  walkOptionBoxJSXClosingElement(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXOpeningElement(pos, ast, visitors) {
  const enterExit = visitors[117];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXOpeningElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);
  walkVecJSXAttributeItem(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXClosingElement(pos, ast, visitors) {
  const enterExit = visitors[118];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXClosingElement(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXFragment(pos, ast, visitors) {
  const enterExit = visitors[119];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXFragment(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXOpeningFragment(pos + 16, ast, visitors);
  walkVecJSXChild(pos + 32, ast, visitors);
  walkJSXClosingFragment(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXOpeningFragment(pos, ast, visitors) {
  const visit = visitors[20];
  if (visit !== null) visit(new JSXOpeningFragment(pos, ast));
}

function walkJSXClosingFragment(pos, ast, visitors) {
  const visit = visitors[21];
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
  const enterExit = visitors[120];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXNamespacedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXIdentifier(pos + 16, ast, visitors);
  walkJSXIdentifier(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXMemberExpression(pos, ast, visitors) {
  const enterExit = visitors[121];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXMemberExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXMemberExpressionObject(pos + 16, ast, visitors);
  walkJSXIdentifier(pos + 32, ast, visitors);

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
  const enterExit = visitors[122];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXExpressionContainer(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXExpression(pos + 16, ast, visitors);

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
      walkBoxSuper(pos + 8, ast, visitors);
      return;
    case 9:
      walkBoxArrayExpression(pos + 8, ast, visitors);
      return;
    case 10:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitors);
      return;
    case 11:
      walkBoxAssignmentExpression(pos + 8, ast, visitors);
      return;
    case 12:
      walkBoxAwaitExpression(pos + 8, ast, visitors);
      return;
    case 13:
      walkBoxBinaryExpression(pos + 8, ast, visitors);
      return;
    case 14:
      walkBoxCallExpression(pos + 8, ast, visitors);
      return;
    case 15:
      walkBoxChainExpression(pos + 8, ast, visitors);
      return;
    case 16:
      walkBoxClass(pos + 8, ast, visitors);
      return;
    case 17:
      walkBoxConditionalExpression(pos + 8, ast, visitors);
      return;
    case 18:
      walkBoxFunction(pos + 8, ast, visitors);
      return;
    case 19:
      walkBoxImportExpression(pos + 8, ast, visitors);
      return;
    case 20:
      walkBoxLogicalExpression(pos + 8, ast, visitors);
      return;
    case 21:
      walkBoxNewExpression(pos + 8, ast, visitors);
      return;
    case 22:
      walkBoxObjectExpression(pos + 8, ast, visitors);
      return;
    case 23:
      walkBoxParenthesizedExpression(pos + 8, ast, visitors);
      return;
    case 24:
      walkBoxSequenceExpression(pos + 8, ast, visitors);
      return;
    case 25:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitors);
      return;
    case 26:
      walkBoxThisExpression(pos + 8, ast, visitors);
      return;
    case 27:
      walkBoxUnaryExpression(pos + 8, ast, visitors);
      return;
    case 28:
      walkBoxUpdateExpression(pos + 8, ast, visitors);
      return;
    case 29:
      walkBoxYieldExpression(pos + 8, ast, visitors);
      return;
    case 30:
      walkBoxPrivateInExpression(pos + 8, ast, visitors);
      return;
    case 31:
      walkBoxImportMeta(pos + 8, ast, visitors);
      return;
    case 32:
      walkBoxNewTarget(pos + 8, ast, visitors);
      return;
    case 33:
      walkBoxJSXElement(pos + 8, ast, visitors);
      return;
    case 34:
      walkBoxJSXFragment(pos + 8, ast, visitors);
      return;
    case 35:
      walkBoxTSAsExpression(pos + 8, ast, visitors);
      return;
    case 36:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitors);
      return;
    case 37:
      walkBoxTSTypeAssertion(pos + 8, ast, visitors);
      return;
    case 38:
      walkBoxTSNonNullExpression(pos + 8, ast, visitors);
      return;
    case 39:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitors);
      return;
    case 40:
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
      walkBoxJSXEmptyExpression(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXExpression`);
  }
}

function walkJSXEmptyExpression(pos, ast, visitors) {
  const visit = visitors[22];
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
  const enterExit = visitors[123];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkJSXAttributeName(pos + 16, ast, visitors);
  walkOptionJSXAttributeValue(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXSpreadAttribute(pos, ast, visitors) {
  const enterExit = visitors[124];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXSpreadAttribute(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

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
  const visit = visitors[23];
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
  const enterExit = visitors[125];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSXSpreadChild(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSXText(pos, ast, visitors) {
  const visit = visitors[24];
  if (visit !== null) visit(new JSXText(pos, ast));
}

function walkTSEnumDeclaration(pos, ast, visitors) {
  const enterExit = visitors[126];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkTSEnumBody(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSEnumBody(pos, ast, visitors) {
  const enterExit = visitors[127];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSEnumMember(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSEnumMember(pos, ast, visitors) {
  const enterExit = visitors[128];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSEnumMember(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSEnumMemberName(pos + 16, ast, visitors);
  walkOptionExpression(pos + 32, ast, visitors);

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
  const enterExit = visitors[129];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAnnotation(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSLiteralType(pos, ast, visitors) {
  const enterExit = visitors[130];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSLiteralType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSLiteral(pos + 16, ast, visitors);

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
  const enterExit = visitors[131];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConditionalType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);
  walkTSType(pos + 32, ast, visitors);
  walkTSType(pos + 48, ast, visitors);
  walkTSType(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSUnionType(pos, ast, visitors) {
  const enterExit = visitors[132];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSUnionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSIntersectionType(pos, ast, visitors) {
  const enterExit = visitors[133];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIntersectionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSParenthesizedType(pos, ast, visitors) {
  const enterExit = visitors[134];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSParenthesizedType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeOperator(pos, ast, visitors) {
  const enterExit = visitors[135];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeOperator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSArrayType(pos, ast, visitors) {
  const enterExit = visitors[136];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSArrayType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSIndexedAccessType(pos, ast, visitors) {
  const enterExit = visitors[137];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexedAccessType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);
  walkTSType(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTupleType(pos, ast, visitors) {
  const enterExit = visitors[138];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTupleType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSTupleElement(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNamedTupleMember(pos, ast, visitors) {
  const enterExit = visitors[139];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamedTupleMember(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 16, ast, visitors);
  walkTSTupleElement(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSOptionalType(pos, ast, visitors) {
  const enterExit = visitors[140];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSOptionalType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSRestType(pos, ast, visitors) {
  const enterExit = visitors[141];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSRestType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

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
  const visit = visitors[25];
  if (visit !== null) visit(new TSAnyKeyword(pos, ast));
}

function walkTSStringKeyword(pos, ast, visitors) {
  const visit = visitors[26];
  if (visit !== null) visit(new TSStringKeyword(pos, ast));
}

function walkTSBooleanKeyword(pos, ast, visitors) {
  const visit = visitors[27];
  if (visit !== null) visit(new TSBooleanKeyword(pos, ast));
}

function walkTSNumberKeyword(pos, ast, visitors) {
  const visit = visitors[28];
  if (visit !== null) visit(new TSNumberKeyword(pos, ast));
}

function walkTSNeverKeyword(pos, ast, visitors) {
  const visit = visitors[29];
  if (visit !== null) visit(new TSNeverKeyword(pos, ast));
}

function walkTSIntrinsicKeyword(pos, ast, visitors) {
  const visit = visitors[30];
  if (visit !== null) visit(new TSIntrinsicKeyword(pos, ast));
}

function walkTSUnknownKeyword(pos, ast, visitors) {
  const visit = visitors[31];
  if (visit !== null) visit(new TSUnknownKeyword(pos, ast));
}

function walkTSNullKeyword(pos, ast, visitors) {
  const visit = visitors[32];
  if (visit !== null) visit(new TSNullKeyword(pos, ast));
}

function walkTSUndefinedKeyword(pos, ast, visitors) {
  const visit = visitors[33];
  if (visit !== null) visit(new TSUndefinedKeyword(pos, ast));
}

function walkTSVoidKeyword(pos, ast, visitors) {
  const visit = visitors[34];
  if (visit !== null) visit(new TSVoidKeyword(pos, ast));
}

function walkTSSymbolKeyword(pos, ast, visitors) {
  const visit = visitors[35];
  if (visit !== null) visit(new TSSymbolKeyword(pos, ast));
}

function walkTSThisType(pos, ast, visitors) {
  const visit = visitors[36];
  if (visit !== null) visit(new TSThisType(pos, ast));
}

function walkTSObjectKeyword(pos, ast, visitors) {
  const visit = visitors[37];
  if (visit !== null) visit(new TSObjectKeyword(pos, ast));
}

function walkTSBigIntKeyword(pos, ast, visitors) {
  const visit = visitors[38];
  if (visit !== null) visit(new TSBigIntKeyword(pos, ast));
}

function walkTSTypeReference(pos, ast, visitors) {
  const enterExit = visitors[142];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeReference(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);

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
  const enterExit = visitors[143];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSQualifiedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 16, ast, visitors);
  walkIdentifierName(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeParameterInstantiation(pos, ast, visitors) {
  const enterExit = visitors[144];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameterInstantiation(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeParameter(pos, ast, visitors) {
  const enterExit = visitors[145];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameter(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkOptionTSType(pos + 48, ast, visitors);
  walkOptionTSType(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeParameterDeclaration(pos, ast, visitors) {
  const enterExit = visitors[146];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeParameterDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSTypeParameter(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeAliasDeclaration(pos, ast, visitors) {
  const enterExit = visitors[147];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAliasDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 48, ast, visitors);
  walkTSType(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSClassImplements(pos, ast, visitors) {
  const enterExit = visitors[148];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSClassImplements(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInterfaceDeclaration(pos, ast, visitors) {
  const enterExit = visitors[149];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 48, ast, visitors);
  walkVecTSInterfaceHeritage(pos + 56, ast, visitors);
  walkBoxTSInterfaceBody(pos + 80, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInterfaceBody(pos, ast, visitors) {
  const enterExit = visitors[150];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceBody(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSPropertySignature(pos, ast, visitors) {
  const enterExit = visitors[151];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSPropertySignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 16, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitors);

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
  const enterExit = visitors[152];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexSignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSIndexSignatureName(pos + 16, ast, visitors);
  walkBoxTSTypeAnnotation(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSCallSignatureDeclaration(pos, ast, visitors) {
  const enterExit = visitors[153];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSCallSignatureDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 16, ast, visitors);
  walkBoxFormalParameters(pos + 32, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSMethodSignature(pos, ast, visitors) {
  const enterExit = visitors[154];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSMethodSignature(pos, ast);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterDeclaration(pos + 32, ast, visitors);
  walkBoxFormalParameters(pos + 48, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 56, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSConstructSignatureDeclaration(pos, ast, visitors) {
  const enterExit = visitors[155];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConstructSignatureDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 16, ast, visitors);
  walkBoxFormalParameters(pos + 24, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSIndexSignatureName(pos, ast, visitors) {
  const enterExit = visitors[156];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSIndexSignatureName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInterfaceHeritage(pos, ast, visitors) {
  const enterExit = visitors[157];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInterfaceHeritage(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypePredicate(pos, ast, visitors) {
  const enterExit = visitors[158];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypePredicate(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypePredicateName(pos + 16, ast, visitors);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypePredicateName(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierName(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSThisType(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypePredicateName`);
  }
}

function walkTSExternalModuleDeclaration(pos, ast, visitors) {
  const enterExit = visitors[159];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExternalModuleDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 16, ast, visitors);
  walkOptionBoxTSModuleBlock(pos + 64, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNamespaceDeclaration(pos, ast, visitors) {
  const enterExit = visitors[160];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamespaceDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkTSNamespaceDeclarationBody(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNamespaceDeclarationBody(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSNamespaceDeclaration(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxTSModuleBlock(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSNamespaceDeclarationBody`);
  }
}

function walkTSGlobalDeclaration(pos, ast, visitors) {
  const enterExit = visitors[161];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSGlobalDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSModuleBlock(pos + 24, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSModuleBlock(pos, ast, visitors) {
  const enterExit = visitors[162];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSModuleBlock(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeLiteral(pos, ast, visitors) {
  const enterExit = visitors[163];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeLiteral(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInferType(pos, ast, visitors) {
  const enterExit = visitors[164];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInferType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeParameter(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeQuery(pos, ast, visitors) {
  const enterExit = visitors[165];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeQuery(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSTypeQueryExprName(pos + 16, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);

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
  const enterExit = visitors[166];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 16, ast, visitors);
  walkOptionBoxObjectExpression(pos + 64, ast, visitors);
  walkOptionTSImportTypeQualifier(pos + 72, ast, visitors);
  walkOptionBoxTSTypeParameterInstantiation(pos + 88, ast, visitors);

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
  const enterExit = visitors[167];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportTypeQualifiedName(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSImportTypeQualifier(pos + 16, ast, visitors);
  walkIdentifierName(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSFunctionType(pos, ast, visitors) {
  const enterExit = visitors[168];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSFunctionType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 16, ast, visitors);
  walkBoxFormalParameters(pos + 32, ast, visitors);
  walkBoxTSTypeAnnotation(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSConstructorType(pos, ast, visitors) {
  const enterExit = visitors[169];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSConstructorType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 16, ast, visitors);
  walkBoxFormalParameters(pos + 24, ast, visitors);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSMappedType(pos, ast, visitors) {
  const enterExit = visitors[170];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSMappedType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkTSType(pos + 48, ast, visitors);
  walkOptionTSType(pos + 64, ast, visitors);
  walkOptionTSType(pos + 80, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTemplateLiteralType(pos, ast, visitors) {
  const enterExit = visitors[171];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTemplateLiteralType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 16, ast, visitors);
  walkVecTSType(pos + 40, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSAsExpression(pos, ast, visitors) {
  const enterExit = visitors[172];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSAsExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkTSType(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSSatisfiesExpression(pos, ast, visitors) {
  const enterExit = visitors[173];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSSatisfiesExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkTSType(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSTypeAssertion(pos, ast, visitors) {
  const enterExit = visitors[174];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSTypeAssertion(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);
  walkExpression(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSImportEqualsDeclaration(pos, ast, visitors) {
  const enterExit = visitors[175];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSImportEqualsDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 16, ast, visitors);
  walkTSModuleReference(pos + 48, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSModuleReference(pos, ast, visitors) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSExternalModuleReference(pos + 8, ast, visitors);
      return;
    case 1:
      walkBoxIdentifierReference(pos + 8, ast, visitors);
      return;
    case 2:
      walkBoxTSQualifiedName(pos + 8, ast, visitors);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleReference`);
  }
}

function walkTSExternalModuleReference(pos, ast, visitors) {
  const enterExit = visitors[176];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExternalModuleReference(pos, ast);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNonNullExpression(pos, ast, visitors) {
  const enterExit = visitors[177];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNonNullExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkDecorator(pos, ast, visitors) {
  const enterExit = visitors[178];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new Decorator(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSExportAssignment(pos, ast, visitors) {
  const enterExit = visitors[179];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSExportAssignment(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSNamespaceExportDeclaration(pos, ast, visitors) {
  const enterExit = visitors[180];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSNamespaceExportDeclaration(pos, ast);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkTSInstantiationExpression(pos, ast, visitors) {
  const enterExit = visitors[181];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new TSInstantiationExpression(pos, ast);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 16, ast, visitors);
  walkBoxTSTypeParameterInstantiation(pos + 32, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSDocNullableType(pos, ast, visitors) {
  const enterExit = visitors[182];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSDocNullableType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSDocNonNullableType(pos, ast, visitors) {
  const enterExit = visitors[183];
  let node,
    enter,
    exit = null;
  if (enterExit !== null) {
    ({ enter, exit } = enterExit);
    node = new JSDocNonNullableType(pos, ast);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 16, ast, visitors);

  if (exit !== null) exit(node);
}

function walkJSDocUnknownType(pos, ast, visitors) {
  const visit = visitors[39];
  if (visit !== null) visit(new JSDocUnknownType(pos, ast));
}

function walkOptionHashbang(pos, ast, visitors) {
  if (!(ast.buffer.int32[(pos >> 2) + 4] === 0 && ast.buffer.int32[(pos >> 2) + 5] === 0))
    walkHashbang(pos, ast, visitors);
}

function walkVecStatement(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkStatement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxBooleanLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkBooleanLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxNullLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkNullLiteral((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxNumericLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkNumericLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxBigIntLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkBigIntLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxRegExpLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkRegExpLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxStringLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkStringLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTemplateLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTemplateLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxIdentifierReference(pos, ast, visitors) {
  const { buffer } = ast;
  return walkIdentifierReference(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxSuper(pos, ast, visitors) {
  const { buffer } = ast;
  return walkSuper((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxArrayExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkArrayExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxArrowFunctionExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkArrowFunctionExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxAssignmentExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAssignmentExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxAwaitExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAwaitExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxBinaryExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkBinaryExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxCallExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkCallExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxChainExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkChainExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxClass(pos, ast, visitors) {
  const { buffer } = ast;
  return walkClass((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxConditionalExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkConditionalExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxFunction(pos, ast, visitors) {
  const { buffer } = ast;
  return walkFunction((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxImportExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkImportExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxLogicalExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkLogicalExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxNewExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkNewExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxObjectExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkObjectExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxParenthesizedExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkParenthesizedExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxSequenceExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkSequenceExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTaggedTemplateExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTaggedTemplateExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxThisExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkThisExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxUnaryExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkUnaryExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxUpdateExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkUpdateExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxYieldExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkYieldExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxPrivateInExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkPrivateInExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxImportMeta(pos, ast, visitors) {
  const { buffer } = ast;
  return walkImportMeta((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxNewTarget(pos, ast, visitors) {
  const { buffer } = ast;
  return walkNewTarget((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxJSXElement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXElement((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxJSXFragment(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXFragment((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSAsExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSAsExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSSatisfiesExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSSatisfiesExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSTypeAssertion(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeAssertion(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNonNullExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNonNullExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSInstantiationExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSInstantiationExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxV8IntrinsicExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkV8IntrinsicExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecArrayExpressionElement(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkArrayExpressionElement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxSpreadElement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkSpreadElement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxElision(pos, ast, visitors) {
  const { buffer } = ast;
  return walkElision((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkVecObjectPropertyKind(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkObjectPropertyKind(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxObjectProperty(pos, ast, visitors) {
  const { buffer } = ast;
  return walkObjectProperty(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxIdentifierName(pos, ast, visitors) {
  const { buffer } = ast;
  return walkIdentifierName(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxPrivateIdentifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkPrivateIdentifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecTemplateElement(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkTemplateElement(pos, ast, visitors);
    pos += 48;
  }
}

function walkVecExpression(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkExpression(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxTSTypeParameterInstantiation(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeParameterInstantiation(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxTSTypeParameterInstantiation(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxTSTypeParameterInstantiation(pos, ast, visitors);
}

function walkBoxComputedMemberExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkComputedMemberExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxStaticMemberExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkStaticMemberExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxPrivateFieldExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkPrivateFieldExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecArgument(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkArgument(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxArrayAssignmentTarget(pos, ast, visitors) {
  const { buffer } = ast;
  return walkArrayAssignmentTarget(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxObjectAssignmentTarget(pos, ast, visitors) {
  const { buffer } = ast;
  return walkObjectAssignmentTarget(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionAssignmentTargetMaybeDefault(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkAssignmentTargetMaybeDefault(pos, ast, visitors);
}

function walkVecOptionAssignmentTargetMaybeDefault(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkOptionAssignmentTargetMaybeDefault(pos, ast, visitors);
    pos += 16;
  }
}

function walkVecAssignmentTargetProperty(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkAssignmentTargetProperty(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxAssignmentTargetWithDefault(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAssignmentTargetWithDefault(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxAssignmentTargetPropertyIdentifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAssignmentTargetPropertyIdentifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxAssignmentTargetPropertyProperty(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAssignmentTargetPropertyProperty(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionExpression(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkExpression(pos, ast, visitors);
}

function walkBoxBlockStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkBlockStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxBreakStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkBreakStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxContinueStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkContinueStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxDebuggerStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkDebuggerStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxDoWhileStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkDoWhileStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxEmptyStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkEmptyStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxExpressionStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExpressionStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxForInStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkForInStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxForOfStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkForOfStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxForStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkForStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxIfStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkIfStatement((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxLabeledStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkLabeledStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxReturnStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkReturnStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxSwitchStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkSwitchStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxThrowStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkThrowStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTryStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTryStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxWhileStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkWhileStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxWithStatement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkWithStatement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxVariableDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkVariableDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSTypeAliasDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeAliasDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSInterfaceDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSInterfaceDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSEnumDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSEnumDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSExternalModuleDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSExternalModuleDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNamespaceDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNamespaceDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSGlobalDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSGlobalDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSImportEqualsDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSImportEqualsDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecVariableDeclarator(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 56;
  while (pos < endPos) {
    walkVariableDeclarator(pos, ast, visitors);
    pos += 56;
  }
}

function walkBoxTSTypeAnnotation(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeAnnotation(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxTSTypeAnnotation(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxTSTypeAnnotation(pos, ast, visitors);
}

function walkOptionStatement(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkStatement(pos, ast, visitors);
}

function walkOptionForStatementInit(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkForStatementInit(pos, ast, visitors);
}

function walkOptionLabelIdentifier(pos, ast, visitors) {
  if (!(ast.buffer.int32[(pos >> 2) + 4] === 0 && ast.buffer.int32[(pos >> 2) + 5] === 0))
    walkLabelIdentifier(pos, ast, visitors);
}

function walkVecSwitchCase(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 56;
  while (pos < endPos) {
    walkSwitchCase(pos, ast, visitors);
    pos += 56;
  }
}

function walkBoxCatchClause(pos, ast, visitors) {
  const { buffer } = ast;
  return walkCatchClause((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkOptionBoxCatchClause(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxCatchClause(pos, ast, visitors);
}

function walkOptionBoxBlockStatement(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxBlockStatement(pos, ast, visitors);
}

function walkOptionCatchParameter(pos, ast, visitors) {
  if (!(ast.buffer[pos + 16] === 255)) walkCatchParameter(pos, ast, visitors);
}

function walkBoxBindingIdentifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkBindingIdentifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxObjectPattern(pos, ast, visitors) {
  const { buffer } = ast;
  return walkObjectPattern(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxArrayPattern(pos, ast, visitors) {
  const { buffer } = ast;
  return walkArrayPattern(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxAssignmentPattern(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAssignmentPattern(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecBindingProperty(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkBindingProperty(pos, ast, visitors);
    pos += 48;
  }
}

function walkOptionBindingPattern(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkBindingPattern(pos, ast, visitors);
}

function walkVecOptionBindingPattern(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkOptionBindingPattern(pos, ast, visitors);
    pos += 16;
  }
}

function walkOptionBindingIdentifier(pos, ast, visitors) {
  if (!(ast.buffer.int32[(pos >> 2) + 4] === 0 && ast.buffer.int32[(pos >> 2) + 5] === 0))
    walkBindingIdentifier(pos, ast, visitors);
}

function walkBoxTSTypeParameterDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeParameterDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxTSTypeParameterDeclaration(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxTSTypeParameterDeclaration(pos, ast, visitors);
}

function walkBoxFormalParameters(pos, ast, visitors) {
  const { buffer } = ast;
  return walkFormalParameters(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxFunctionBody(pos, ast, visitors) {
  const { buffer } = ast;
  return walkFunctionBody(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxFunctionBody(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxFunctionBody(pos, ast, visitors);
}

function walkVecFormalParameter(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 72;
  while (pos < endPos) {
    walkFormalParameter(pos, ast, visitors);
    pos += 72;
  }
}

function walkVecDecorator(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkDecorator(pos, ast, visitors);
    pos += 32;
  }
}

function walkBoxExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExpression((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkOptionBoxExpression(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxExpression(pos, ast, visitors);
}

function walkVecTSClassImplements(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 40;
  while (pos < endPos) {
    walkTSClassImplements(pos, ast, visitors);
    pos += 40;
  }
}

function walkBoxClassBody(pos, ast, visitors) {
  const { buffer } = ast;
  return walkClassBody((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkVecClassElement(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkClassElement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxStaticBlock(pos, ast, visitors) {
  const { buffer } = ast;
  return walkStaticBlock((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxMethodDefinition(pos, ast, visitors) {
  const { buffer } = ast;
  return walkMethodDefinition(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxPropertyDefinition(pos, ast, visitors) {
  const { buffer } = ast;
  return walkPropertyDefinition(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxAccessorProperty(pos, ast, visitors) {
  const { buffer } = ast;
  return walkAccessorProperty(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSIndexSignature(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSIndexSignature(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxImportDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkImportDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxExportAllDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExportAllDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxExportDefaultDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExportDefaultDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxExportDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExportDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxExportNamedDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExportNamedDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxExportFromDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkExportFromDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSExportAssignment(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSExportAssignment(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNamespaceExportDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNamespaceExportDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecImportDeclarationSpecifier(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkImportDeclarationSpecifier(pos, ast, visitors);
    pos += 16;
  }
}

function walkOptionVecImportDeclarationSpecifier(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkVecImportDeclarationSpecifier(pos, ast, visitors);
}

function walkBoxWithClause(pos, ast, visitors) {
  const { buffer } = ast;
  return walkWithClause((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkOptionBoxWithClause(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxWithClause(pos, ast, visitors);
}

function walkBoxImportSpecifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkImportSpecifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxImportDefaultSpecifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkImportDefaultSpecifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxImportNamespaceSpecifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkImportNamespaceSpecifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecImportAttribute(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 120;
  while (pos < endPos) {
    walkImportAttribute(pos, ast, visitors);
    pos += 120;
  }
}

function walkVecExportSpecifier(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 128;
  while (pos < endPos) {
    walkExportSpecifier(pos, ast, visitors);
    pos += 128;
  }
}

function walkOptionModuleExportName(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkModuleExportName(pos, ast, visitors);
}

function walkBoxJSXOpeningElement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXOpeningElement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecJSXChild(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkJSXChild(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxJSXClosingElement(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXClosingElement(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxJSXClosingElement(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxJSXClosingElement(pos, ast, visitors);
}

function walkVecJSXAttributeItem(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkJSXAttributeItem(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxJSXIdentifier(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXIdentifier(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSXNamespacedName(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXNamespacedName(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSXMemberExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXMemberExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSXEmptyExpression(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXEmptyExpression(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSXAttribute(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXAttribute(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSXSpreadAttribute(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXSpreadAttribute(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionJSXAttributeValue(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkJSXAttributeValue(pos, ast, visitors);
}

function walkBoxJSXExpressionContainer(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXExpressionContainer(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSXText(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXText((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxJSXSpreadChild(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSXSpreadChild(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecTSEnumMember(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkTSEnumMember(pos, ast, visitors);
    pos += 48;
  }
}

function walkBoxTSAnyKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSAnyKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSBigIntKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSBigIntKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSBooleanKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSBooleanKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSIntrinsicKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSIntrinsicKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNeverKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNeverKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNullKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNullKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNumberKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNumberKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSObjectKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSObjectKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSStringKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSStringKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSSymbolKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSSymbolKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSUndefinedKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSUndefinedKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSUnknownKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSUnknownKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSVoidKeyword(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSVoidKeyword(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSArrayType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSArrayType((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSConditionalType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSConditionalType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSConstructorType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSConstructorType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSFunctionType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSFunctionType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSImportType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSImportType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSIndexedAccessType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSIndexedAccessType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSInferType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSInferType((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSIntersectionType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSIntersectionType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSLiteralType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSLiteralType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSMappedType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSMappedType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSNamedTupleMember(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSNamedTupleMember(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSTemplateLiteralType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTemplateLiteralType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSThisType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSThisType((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSTupleType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTupleType((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSTypeLiteral(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeLiteral(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSTypeOperator(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeOperator(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSTypePredicate(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypePredicate(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSTypeQuery(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeQuery((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSTypeReference(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeReference(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSUnionType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSUnionType((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSParenthesizedType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSParenthesizedType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSDocNullableType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSDocNullableType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSDocNonNullableType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSDocNonNullableType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxJSDocUnknownType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkJSDocUnknownType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecTSType(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSType(pos, ast, visitors);
    pos += 16;
  }
}

function walkVecTSTupleElement(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSTupleElement(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxTSOptionalType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSOptionalType(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSRestType(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSRestType((buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase, ast, visitors);
}

function walkBoxTSQualifiedName(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSQualifiedName(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionTSType(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 255)) walkTSType(pos, ast, visitors);
}

function walkVecTSTypeParameter(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 80;
  while (pos < endPos) {
    walkTSTypeParameter(pos, ast, visitors);
    pos += 80;
  }
}

function walkVecTSInterfaceHeritage(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 40;
  while (pos < endPos) {
    walkTSInterfaceHeritage(pos, ast, visitors);
    pos += 40;
  }
}

function walkBoxTSInterfaceBody(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSInterfaceBody(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkVecTSSignature(pos, ast, visitors) {
  const { int32, ptrFlip, ptrBase } = ast.buffer,
    pos32 = pos >> 2;
  pos = (int32[pos32] ^ ptrFlip) - ptrBase;
  const endPos = pos + int32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSSignature(pos, ast, visitors);
    pos += 16;
  }
}

function walkBoxTSPropertySignature(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSPropertySignature(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSCallSignatureDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSCallSignatureDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSConstructSignatureDeclaration(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSConstructSignatureDeclaration(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSMethodSignature(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSMethodSignature(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSModuleBlock(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSModuleBlock(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxTSModuleBlock(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxTSModuleBlock(pos, ast, visitors);
}

function walkBoxTSTypeParameter(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSTypeParameter(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkOptionBoxObjectExpression(pos, ast, visitors) {
  if (!(ast.buffer.int32[pos >> 2] === 0 && ast.buffer.int32[(pos >> 2) + 1] === 0))
    walkBoxObjectExpression(pos, ast, visitors);
}

function walkOptionTSImportTypeQualifier(pos, ast, visitors) {
  if (!(ast.buffer[pos] === 2)) walkTSImportTypeQualifier(pos, ast, visitors);
}

function walkBoxTSImportTypeQualifiedName(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSImportTypeQualifiedName(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}

function walkBoxTSExternalModuleReference(pos, ast, visitors) {
  const { buffer } = ast;
  return walkTSExternalModuleReference(
    (buffer.int32[pos >> 2] ^ buffer.ptrFlip) - buffer.ptrBase,
    ast,
    visitors,
  );
}
