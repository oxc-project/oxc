// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/estree_visit.rs`.

export { walkProgram };

const { isArray } = Array;

function walkNode(node, visitors) {
  if (node == null) return;
  if (isArray(node)) {
    const len = node.length;
    for (let i = 0; i < len; i++) {
      walkNode(node[i], visitors);
    }
  } else {
    switch (node.type) {
      case 'DebuggerStatement':
        walkDebuggerStatement(node, visitors);
        break;
      case 'EmptyStatement':
        walkEmptyStatement(node, visitors);
        break;
      case 'Literal':
        walkLiteral(node, visitors);
        break;
      case 'PrivateIdentifier':
        walkPrivateIdentifier(node, visitors);
        break;
      case 'Super':
        walkSuper(node, visitors);
        break;
      case 'TemplateElement':
        walkTemplateElement(node, visitors);
        break;
      case 'ThisExpression':
        walkThisExpression(node, visitors);
        break;
      case 'JSXClosingFragment':
        walkJSXClosingFragment(node, visitors);
        break;
      case 'JSXEmptyExpression':
        walkJSXEmptyExpression(node, visitors);
        break;
      case 'JSXIdentifier':
        walkJSXIdentifier(node, visitors);
        break;
      case 'JSXOpeningFragment':
        walkJSXOpeningFragment(node, visitors);
        break;
      case 'JSXText':
        walkJSXText(node, visitors);
        break;
      case 'TSAnyKeyword':
        walkTSAnyKeyword(node, visitors);
        break;
      case 'TSBigIntKeyword':
        walkTSBigIntKeyword(node, visitors);
        break;
      case 'TSBooleanKeyword':
        walkTSBooleanKeyword(node, visitors);
        break;
      case 'TSIntrinsicKeyword':
        walkTSIntrinsicKeyword(node, visitors);
        break;
      case 'TSJSDocUnknownType':
        walkTSJSDocUnknownType(node, visitors);
        break;
      case 'TSNeverKeyword':
        walkTSNeverKeyword(node, visitors);
        break;
      case 'TSNullKeyword':
        walkTSNullKeyword(node, visitors);
        break;
      case 'TSNumberKeyword':
        walkTSNumberKeyword(node, visitors);
        break;
      case 'TSObjectKeyword':
        walkTSObjectKeyword(node, visitors);
        break;
      case 'TSStringKeyword':
        walkTSStringKeyword(node, visitors);
        break;
      case 'TSSymbolKeyword':
        walkTSSymbolKeyword(node, visitors);
        break;
      case 'TSThisType':
        walkTSThisType(node, visitors);
        break;
      case 'TSUndefinedKeyword':
        walkTSUndefinedKeyword(node, visitors);
        break;
      case 'TSUnknownKeyword':
        walkTSUnknownKeyword(node, visitors);
        break;
      case 'TSVoidKeyword':
        walkTSVoidKeyword(node, visitors);
        break;
      case 'AccessorProperty':
        walkAccessorProperty(node, visitors);
        break;
      case 'ArrayExpression':
        walkArrayExpression(node, visitors);
        break;
      case 'ArrayPattern':
        walkArrayPattern(node, visitors);
        break;
      case 'ArrowFunctionExpression':
        walkArrowFunctionExpression(node, visitors);
        break;
      case 'AssignmentExpression':
        walkAssignmentExpression(node, visitors);
        break;
      case 'AssignmentPattern':
        walkAssignmentPattern(node, visitors);
        break;
      case 'AwaitExpression':
        walkAwaitExpression(node, visitors);
        break;
      case 'BinaryExpression':
        walkBinaryExpression(node, visitors);
        break;
      case 'BlockStatement':
        walkBlockStatement(node, visitors);
        break;
      case 'BreakStatement':
        walkBreakStatement(node, visitors);
        break;
      case 'CallExpression':
        walkCallExpression(node, visitors);
        break;
      case 'CatchClause':
        walkCatchClause(node, visitors);
        break;
      case 'ChainExpression':
        walkChainExpression(node, visitors);
        break;
      case 'ClassBody':
        walkClassBody(node, visitors);
        break;
      case 'ClassDeclaration':
        walkClassDeclaration(node, visitors);
        break;
      case 'ClassExpression':
        walkClassExpression(node, visitors);
        break;
      case 'ConditionalExpression':
        walkConditionalExpression(node, visitors);
        break;
      case 'ContinueStatement':
        walkContinueStatement(node, visitors);
        break;
      case 'Decorator':
        walkDecorator(node, visitors);
        break;
      case 'DoWhileStatement':
        walkDoWhileStatement(node, visitors);
        break;
      case 'ExportAllDeclaration':
        walkExportAllDeclaration(node, visitors);
        break;
      case 'ExportDefaultDeclaration':
        walkExportDefaultDeclaration(node, visitors);
        break;
      case 'ExportNamedDeclaration':
        walkExportNamedDeclaration(node, visitors);
        break;
      case 'ExportSpecifier':
        walkExportSpecifier(node, visitors);
        break;
      case 'ExpressionStatement':
        walkExpressionStatement(node, visitors);
        break;
      case 'ForInStatement':
        walkForInStatement(node, visitors);
        break;
      case 'ForOfStatement':
        walkForOfStatement(node, visitors);
        break;
      case 'ForStatement':
        walkForStatement(node, visitors);
        break;
      case 'FunctionDeclaration':
        walkFunctionDeclaration(node, visitors);
        break;
      case 'FunctionExpression':
        walkFunctionExpression(node, visitors);
        break;
      case 'Identifier':
        walkIdentifier(node, visitors);
        break;
      case 'IfStatement':
        walkIfStatement(node, visitors);
        break;
      case 'ImportAttribute':
        walkImportAttribute(node, visitors);
        break;
      case 'ImportDeclaration':
        walkImportDeclaration(node, visitors);
        break;
      case 'ImportDefaultSpecifier':
        walkImportDefaultSpecifier(node, visitors);
        break;
      case 'ImportExpression':
        walkImportExpression(node, visitors);
        break;
      case 'ImportNamespaceSpecifier':
        walkImportNamespaceSpecifier(node, visitors);
        break;
      case 'ImportSpecifier':
        walkImportSpecifier(node, visitors);
        break;
      case 'LabeledStatement':
        walkLabeledStatement(node, visitors);
        break;
      case 'LogicalExpression':
        walkLogicalExpression(node, visitors);
        break;
      case 'MemberExpression':
        walkMemberExpression(node, visitors);
        break;
      case 'MetaProperty':
        walkMetaProperty(node, visitors);
        break;
      case 'MethodDefinition':
        walkMethodDefinition(node, visitors);
        break;
      case 'NewExpression':
        walkNewExpression(node, visitors);
        break;
      case 'ObjectExpression':
        walkObjectExpression(node, visitors);
        break;
      case 'ObjectPattern':
        walkObjectPattern(node, visitors);
        break;
      case 'ParenthesizedExpression':
        walkParenthesizedExpression(node, visitors);
        break;
      case 'Program':
        walkProgram(node, visitors);
        break;
      case 'Property':
        walkProperty(node, visitors);
        break;
      case 'PropertyDefinition':
        walkPropertyDefinition(node, visitors);
        break;
      case 'RestElement':
        walkRestElement(node, visitors);
        break;
      case 'ReturnStatement':
        walkReturnStatement(node, visitors);
        break;
      case 'SequenceExpression':
        walkSequenceExpression(node, visitors);
        break;
      case 'SpreadElement':
        walkSpreadElement(node, visitors);
        break;
      case 'StaticBlock':
        walkStaticBlock(node, visitors);
        break;
      case 'SwitchCase':
        walkSwitchCase(node, visitors);
        break;
      case 'SwitchStatement':
        walkSwitchStatement(node, visitors);
        break;
      case 'TaggedTemplateExpression':
        walkTaggedTemplateExpression(node, visitors);
        break;
      case 'TemplateLiteral':
        walkTemplateLiteral(node, visitors);
        break;
      case 'ThrowStatement':
        walkThrowStatement(node, visitors);
        break;
      case 'TryStatement':
        walkTryStatement(node, visitors);
        break;
      case 'UnaryExpression':
        walkUnaryExpression(node, visitors);
        break;
      case 'UpdateExpression':
        walkUpdateExpression(node, visitors);
        break;
      case 'V8IntrinsicExpression':
        walkV8IntrinsicExpression(node, visitors);
        break;
      case 'VariableDeclaration':
        walkVariableDeclaration(node, visitors);
        break;
      case 'VariableDeclarator':
        walkVariableDeclarator(node, visitors);
        break;
      case 'WhileStatement':
        walkWhileStatement(node, visitors);
        break;
      case 'WithStatement':
        walkWithStatement(node, visitors);
        break;
      case 'YieldExpression':
        walkYieldExpression(node, visitors);
        break;
      case 'JSXAttribute':
        walkJSXAttribute(node, visitors);
        break;
      case 'JSXClosingElement':
        walkJSXClosingElement(node, visitors);
        break;
      case 'JSXElement':
        walkJSXElement(node, visitors);
        break;
      case 'JSXExpressionContainer':
        walkJSXExpressionContainer(node, visitors);
        break;
      case 'JSXFragment':
        walkJSXFragment(node, visitors);
        break;
      case 'JSXMemberExpression':
        walkJSXMemberExpression(node, visitors);
        break;
      case 'JSXNamespacedName':
        walkJSXNamespacedName(node, visitors);
        break;
      case 'JSXOpeningElement':
        walkJSXOpeningElement(node, visitors);
        break;
      case 'JSXSpreadAttribute':
        walkJSXSpreadAttribute(node, visitors);
        break;
      case 'JSXSpreadChild':
        walkJSXSpreadChild(node, visitors);
        break;
      case 'TSAbstractAccessorProperty':
        walkTSAbstractAccessorProperty(node, visitors);
        break;
      case 'TSAbstractMethodDefinition':
        walkTSAbstractMethodDefinition(node, visitors);
        break;
      case 'TSAbstractPropertyDefinition':
        walkTSAbstractPropertyDefinition(node, visitors);
        break;
      case 'TSArrayType':
        walkTSArrayType(node, visitors);
        break;
      case 'TSAsExpression':
        walkTSAsExpression(node, visitors);
        break;
      case 'TSCallSignatureDeclaration':
        walkTSCallSignatureDeclaration(node, visitors);
        break;
      case 'TSClassImplements':
        walkTSClassImplements(node, visitors);
        break;
      case 'TSConditionalType':
        walkTSConditionalType(node, visitors);
        break;
      case 'TSConstructSignatureDeclaration':
        walkTSConstructSignatureDeclaration(node, visitors);
        break;
      case 'TSConstructorType':
        walkTSConstructorType(node, visitors);
        break;
      case 'TSDeclareFunction':
        walkTSDeclareFunction(node, visitors);
        break;
      case 'TSEmptyBodyFunctionExpression':
        walkTSEmptyBodyFunctionExpression(node, visitors);
        break;
      case 'TSEnumBody':
        walkTSEnumBody(node, visitors);
        break;
      case 'TSEnumDeclaration':
        walkTSEnumDeclaration(node, visitors);
        break;
      case 'TSEnumMember':
        walkTSEnumMember(node, visitors);
        break;
      case 'TSExportAssignment':
        walkTSExportAssignment(node, visitors);
        break;
      case 'TSExternalModuleReference':
        walkTSExternalModuleReference(node, visitors);
        break;
      case 'TSFunctionType':
        walkTSFunctionType(node, visitors);
        break;
      case 'TSImportEqualsDeclaration':
        walkTSImportEqualsDeclaration(node, visitors);
        break;
      case 'TSImportType':
        walkTSImportType(node, visitors);
        break;
      case 'TSIndexSignature':
        walkTSIndexSignature(node, visitors);
        break;
      case 'TSIndexedAccessType':
        walkTSIndexedAccessType(node, visitors);
        break;
      case 'TSInferType':
        walkTSInferType(node, visitors);
        break;
      case 'TSInstantiationExpression':
        walkTSInstantiationExpression(node, visitors);
        break;
      case 'TSInterfaceBody':
        walkTSInterfaceBody(node, visitors);
        break;
      case 'TSInterfaceDeclaration':
        walkTSInterfaceDeclaration(node, visitors);
        break;
      case 'TSInterfaceHeritage':
        walkTSInterfaceHeritage(node, visitors);
        break;
      case 'TSIntersectionType':
        walkTSIntersectionType(node, visitors);
        break;
      case 'TSJSDocNonNullableType':
        walkTSJSDocNonNullableType(node, visitors);
        break;
      case 'TSJSDocNullableType':
        walkTSJSDocNullableType(node, visitors);
        break;
      case 'TSLiteralType':
        walkTSLiteralType(node, visitors);
        break;
      case 'TSMappedType':
        walkTSMappedType(node, visitors);
        break;
      case 'TSMethodSignature':
        walkTSMethodSignature(node, visitors);
        break;
      case 'TSModuleBlock':
        walkTSModuleBlock(node, visitors);
        break;
      case 'TSModuleDeclaration':
        walkTSModuleDeclaration(node, visitors);
        break;
      case 'TSNamedTupleMember':
        walkTSNamedTupleMember(node, visitors);
        break;
      case 'TSNamespaceExportDeclaration':
        walkTSNamespaceExportDeclaration(node, visitors);
        break;
      case 'TSNonNullExpression':
        walkTSNonNullExpression(node, visitors);
        break;
      case 'TSOptionalType':
        walkTSOptionalType(node, visitors);
        break;
      case 'TSParameterProperty':
        walkTSParameterProperty(node, visitors);
        break;
      case 'TSParenthesizedType':
        walkTSParenthesizedType(node, visitors);
        break;
      case 'TSPropertySignature':
        walkTSPropertySignature(node, visitors);
        break;
      case 'TSQualifiedName':
        walkTSQualifiedName(node, visitors);
        break;
      case 'TSRestType':
        walkTSRestType(node, visitors);
        break;
      case 'TSSatisfiesExpression':
        walkTSSatisfiesExpression(node, visitors);
        break;
      case 'TSTemplateLiteralType':
        walkTSTemplateLiteralType(node, visitors);
        break;
      case 'TSTupleType':
        walkTSTupleType(node, visitors);
        break;
      case 'TSTypeAliasDeclaration':
        walkTSTypeAliasDeclaration(node, visitors);
        break;
      case 'TSTypeAnnotation':
        walkTSTypeAnnotation(node, visitors);
        break;
      case 'TSTypeAssertion':
        walkTSTypeAssertion(node, visitors);
        break;
      case 'TSTypeLiteral':
        walkTSTypeLiteral(node, visitors);
        break;
      case 'TSTypeOperator':
        walkTSTypeOperator(node, visitors);
        break;
      case 'TSTypeParameter':
        walkTSTypeParameter(node, visitors);
        break;
      case 'TSTypeParameterDeclaration':
        walkTSTypeParameterDeclaration(node, visitors);
        break;
      case 'TSTypeParameterInstantiation':
        walkTSTypeParameterInstantiation(node, visitors);
        break;
      case 'TSTypePredicate':
        walkTSTypePredicate(node, visitors);
        break;
      case 'TSTypeQuery':
        walkTSTypeQuery(node, visitors);
        break;
      case 'TSTypeReference':
        walkTSTypeReference(node, visitors);
        break;
      case 'TSUnionType':
        walkTSUnionType(node, visitors);
        break;
    }
  }
}

function walkDebuggerStatement(node, visitors) {
  const visit = visitors[0];
  if (visit !== null) visit(node);
}

function walkEmptyStatement(node, visitors) {
  const visit = visitors[1];
  if (visit !== null) visit(node);
}

function walkLiteral(node, visitors) {
  const visit = visitors[2];
  if (visit !== null) visit(node);
}

function walkPrivateIdentifier(node, visitors) {
  const visit = visitors[3];
  if (visit !== null) visit(node);
}

function walkSuper(node, visitors) {
  const visit = visitors[4];
  if (visit !== null) visit(node);
}

function walkTemplateElement(node, visitors) {
  const visit = visitors[5];
  if (visit !== null) visit(node);
}

function walkThisExpression(node, visitors) {
  const visit = visitors[6];
  if (visit !== null) visit(node);
}

function walkJSXClosingFragment(node, visitors) {
  const visit = visitors[7];
  if (visit !== null) visit(node);
}

function walkJSXEmptyExpression(node, visitors) {
  const visit = visitors[8];
  if (visit !== null) visit(node);
}

function walkJSXIdentifier(node, visitors) {
  const visit = visitors[9];
  if (visit !== null) visit(node);
}

function walkJSXOpeningFragment(node, visitors) {
  const visit = visitors[10];
  if (visit !== null) visit(node);
}

function walkJSXText(node, visitors) {
  const visit = visitors[11];
  if (visit !== null) visit(node);
}

function walkTSAnyKeyword(node, visitors) {
  const visit = visitors[12];
  if (visit !== null) visit(node);
}

function walkTSBigIntKeyword(node, visitors) {
  const visit = visitors[13];
  if (visit !== null) visit(node);
}

function walkTSBooleanKeyword(node, visitors) {
  const visit = visitors[14];
  if (visit !== null) visit(node);
}

function walkTSIntrinsicKeyword(node, visitors) {
  const visit = visitors[15];
  if (visit !== null) visit(node);
}

function walkTSJSDocUnknownType(node, visitors) {
  const visit = visitors[16];
  if (visit !== null) visit(node);
}

function walkTSNeverKeyword(node, visitors) {
  const visit = visitors[17];
  if (visit !== null) visit(node);
}

function walkTSNullKeyword(node, visitors) {
  const visit = visitors[18];
  if (visit !== null) visit(node);
}

function walkTSNumberKeyword(node, visitors) {
  const visit = visitors[19];
  if (visit !== null) visit(node);
}

function walkTSObjectKeyword(node, visitors) {
  const visit = visitors[20];
  if (visit !== null) visit(node);
}

function walkTSStringKeyword(node, visitors) {
  const visit = visitors[21];
  if (visit !== null) visit(node);
}

function walkTSSymbolKeyword(node, visitors) {
  const visit = visitors[22];
  if (visit !== null) visit(node);
}

function walkTSThisType(node, visitors) {
  const visit = visitors[23];
  if (visit !== null) visit(node);
}

function walkTSUndefinedKeyword(node, visitors) {
  const visit = visitors[24];
  if (visit !== null) visit(node);
}

function walkTSUnknownKeyword(node, visitors) {
  const visit = visitors[25];
  if (visit !== null) visit(node);
}

function walkTSVoidKeyword(node, visitors) {
  const visit = visitors[26];
  if (visit !== null) visit(node);
}

function walkAccessorProperty(node, visitors) {
  const enterExit = visitors[27];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkArrayExpression(node, visitors) {
  const enterExit = visitors[28];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.elements, visitors);
  if (exit !== null) exit(node);
}

function walkArrayPattern(node, visitors) {
  const enterExit = visitors[29];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.elements, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkArrowFunctionExpression(node, visitors) {
  const enterExit = visitors[30];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkAssignmentExpression(node, visitors) {
  const enterExit = visitors[31];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  if (exit !== null) exit(node);
}

function walkAssignmentPattern(node, visitors) {
  const enterExit = visitors[32];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkAwaitExpression(node, visitors) {
  const enterExit = visitors[33];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkBinaryExpression(node, visitors) {
  const enterExit = visitors[34];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  if (exit !== null) exit(node);
}

function walkBlockStatement(node, visitors) {
  const enterExit = visitors[35];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkBreakStatement(node, visitors) {
  const enterExit = visitors[36];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.label, visitors);
  if (exit !== null) exit(node);
}

function walkCallExpression(node, visitors) {
  const enterExit = visitors[37];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.callee, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.arguments, visitors);
  if (exit !== null) exit(node);
}

function walkCatchClause(node, visitors) {
  const enterExit = visitors[38];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.param, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkChainExpression(node, visitors) {
  const enterExit = visitors[39];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkClassBody(node, visitors) {
  const enterExit = visitors[40];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkClassDeclaration(node, visitors) {
  const enterExit = visitors[41];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.superClass, visitors);
  walkNode(node.superTypeArguments, visitors);
  walkNode(node.implements, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkClassExpression(node, visitors) {
  const enterExit = visitors[42];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.superClass, visitors);
  walkNode(node.superTypeArguments, visitors);
  walkNode(node.implements, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkConditionalExpression(node, visitors) {
  const enterExit = visitors[43];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.consequent, visitors);
  walkNode(node.alternate, visitors);
  if (exit !== null) exit(node);
}

function walkContinueStatement(node, visitors) {
  const enterExit = visitors[44];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.label, visitors);
  if (exit !== null) exit(node);
}

function walkDecorator(node, visitors) {
  const enterExit = visitors[45];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkDoWhileStatement(node, visitors) {
  const enterExit = visitors[46];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  walkNode(node.test, visitors);
  if (exit !== null) exit(node);
}

function walkExportAllDeclaration(node, visitors) {
  const enterExit = visitors[47];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.exported, visitors);
  walkNode(node.source, visitors);
  walkNode(node.attributes, visitors);
  if (exit !== null) exit(node);
}

function walkExportDefaultDeclaration(node, visitors) {
  const enterExit = visitors[48];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.declaration, visitors);
  if (exit !== null) exit(node);
}

function walkExportNamedDeclaration(node, visitors) {
  const enterExit = visitors[49];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.declaration, visitors);
  walkNode(node.specifiers, visitors);
  walkNode(node.source, visitors);
  walkNode(node.attributes, visitors);
  if (exit !== null) exit(node);
}

function walkExportSpecifier(node, visitors) {
  const enterExit = visitors[50];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.local, visitors);
  walkNode(node.exported, visitors);
  if (exit !== null) exit(node);
}

function walkExpressionStatement(node, visitors) {
  const enterExit = visitors[51];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkForInStatement(node, visitors) {
  const enterExit = visitors[52];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkForOfStatement(node, visitors) {
  const enterExit = visitors[53];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkForStatement(node, visitors) {
  const enterExit = visitors[54];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.init, visitors);
  walkNode(node.test, visitors);
  walkNode(node.update, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkFunctionDeclaration(node, visitors) {
  const enterExit = visitors[55];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkFunctionExpression(node, visitors) {
  const enterExit = visitors[56];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkIdentifier(node, visitors) {
  const enterExit = visitors[57];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkIfStatement(node, visitors) {
  const enterExit = visitors[58];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.consequent, visitors);
  walkNode(node.alternate, visitors);
  if (exit !== null) exit(node);
}

function walkImportAttribute(node, visitors) {
  const enterExit = visitors[59];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkImportDeclaration(node, visitors) {
  const enterExit = visitors[60];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.specifiers, visitors);
  walkNode(node.source, visitors);
  walkNode(node.attributes, visitors);
  if (exit !== null) exit(node);
}

function walkImportDefaultSpecifier(node, visitors) {
  const enterExit = visitors[61];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.local, visitors);
  if (exit !== null) exit(node);
}

function walkImportExpression(node, visitors) {
  const enterExit = visitors[62];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.source, visitors);
  walkNode(node.options, visitors);
  if (exit !== null) exit(node);
}

function walkImportNamespaceSpecifier(node, visitors) {
  const enterExit = visitors[63];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.local, visitors);
  if (exit !== null) exit(node);
}

function walkImportSpecifier(node, visitors) {
  const enterExit = visitors[64];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.imported, visitors);
  walkNode(node.local, visitors);
  if (exit !== null) exit(node);
}

function walkLabeledStatement(node, visitors) {
  const enterExit = visitors[65];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.label, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkLogicalExpression(node, visitors) {
  const enterExit = visitors[66];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  if (exit !== null) exit(node);
}

function walkMemberExpression(node, visitors) {
  const enterExit = visitors[67];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.object, visitors);
  walkNode(node.property, visitors);
  if (exit !== null) exit(node);
}

function walkMetaProperty(node, visitors) {
  const enterExit = visitors[68];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.meta, visitors);
  walkNode(node.property, visitors);
  if (exit !== null) exit(node);
}

function walkMethodDefinition(node, visitors) {
  const enterExit = visitors[69];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkNewExpression(node, visitors) {
  const enterExit = visitors[70];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.callee, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.arguments, visitors);
  if (exit !== null) exit(node);
}

function walkObjectExpression(node, visitors) {
  const enterExit = visitors[71];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.properties, visitors);
  if (exit !== null) exit(node);
}

function walkObjectPattern(node, visitors) {
  const enterExit = visitors[72];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.properties, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkParenthesizedExpression(node, visitors) {
  const enterExit = visitors[73];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkProgram(node, visitors) {
  const enterExit = visitors[74];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkProperty(node, visitors) {
  const enterExit = visitors[75];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkPropertyDefinition(node, visitors) {
  const enterExit = visitors[76];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkRestElement(node, visitors) {
  const enterExit = visitors[77];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.argument, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkReturnStatement(node, visitors) {
  const enterExit = visitors[78];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkSequenceExpression(node, visitors) {
  const enterExit = visitors[79];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expressions, visitors);
  if (exit !== null) exit(node);
}

function walkSpreadElement(node, visitors) {
  const enterExit = visitors[80];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkStaticBlock(node, visitors) {
  const enterExit = visitors[81];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkSwitchCase(node, visitors) {
  const enterExit = visitors[82];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.consequent, visitors);
  if (exit !== null) exit(node);
}

function walkSwitchStatement(node, visitors) {
  const enterExit = visitors[83];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.discriminant, visitors);
  walkNode(node.cases, visitors);
  if (exit !== null) exit(node);
}

function walkTaggedTemplateExpression(node, visitors) {
  const enterExit = visitors[84];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.tag, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.quasi, visitors);
  if (exit !== null) exit(node);
}

function walkTemplateLiteral(node, visitors) {
  const enterExit = visitors[85];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.quasis, visitors);
  walkNode(node.expressions, visitors);
  if (exit !== null) exit(node);
}

function walkThrowStatement(node, visitors) {
  const enterExit = visitors[86];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkTryStatement(node, visitors) {
  const enterExit = visitors[87];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.block, visitors);
  walkNode(node.handler, visitors);
  walkNode(node.finalizer, visitors);
  if (exit !== null) exit(node);
}

function walkUnaryExpression(node, visitors) {
  const enterExit = visitors[88];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkUpdateExpression(node, visitors) {
  const enterExit = visitors[89];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkV8IntrinsicExpression(node, visitors) {
  const enterExit = visitors[90];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.arguments, visitors);
  if (exit !== null) exit(node);
}

function walkVariableDeclaration(node, visitors) {
  const enterExit = visitors[91];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.declarations, visitors);
  if (exit !== null) exit(node);
}

function walkVariableDeclarator(node, visitors) {
  const enterExit = visitors[92];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.init, visitors);
  if (exit !== null) exit(node);
}

function walkWhileStatement(node, visitors) {
  const enterExit = visitors[93];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkWithStatement(node, visitors) {
  const enterExit = visitors[94];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.object, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkYieldExpression(node, visitors) {
  const enterExit = visitors[95];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkJSXAttribute(node, visitors) {
  const enterExit = visitors[96];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkJSXClosingElement(node, visitors) {
  const enterExit = visitors[97];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.name, visitors);
  if (exit !== null) exit(node);
}

function walkJSXElement(node, visitors) {
  const enterExit = visitors[98];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.openingElement, visitors);
  walkNode(node.children, visitors);
  walkNode(node.closingElement, visitors);
  if (exit !== null) exit(node);
}

function walkJSXExpressionContainer(node, visitors) {
  const enterExit = visitors[99];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkJSXFragment(node, visitors) {
  const enterExit = visitors[100];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.openingFragment, visitors);
  walkNode(node.children, visitors);
  walkNode(node.closingFragment, visitors);
  if (exit !== null) exit(node);
}

function walkJSXMemberExpression(node, visitors) {
  const enterExit = visitors[101];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.object, visitors);
  walkNode(node.property, visitors);
  if (exit !== null) exit(node);
}

function walkJSXNamespacedName(node, visitors) {
  const enterExit = visitors[102];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.namespace, visitors);
  walkNode(node.name, visitors);
  if (exit !== null) exit(node);
}

function walkJSXOpeningElement(node, visitors) {
  const enterExit = visitors[103];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.attributes, visitors);
  if (exit !== null) exit(node);
}

function walkJSXSpreadAttribute(node, visitors) {
  const enterExit = visitors[104];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  if (exit !== null) exit(node);
}

function walkJSXSpreadChild(node, visitors) {
  const enterExit = visitors[105];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkTSAbstractAccessorProperty(node, visitors) {
  const enterExit = visitors[106];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSAbstractMethodDefinition(node, visitors) {
  const enterExit = visitors[107];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  if (exit !== null) exit(node);
}

function walkTSAbstractPropertyDefinition(node, visitors) {
  const enterExit = visitors[108];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSArrayType(node, visitors) {
  const enterExit = visitors[109];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.elementType, visitors);
  if (exit !== null) exit(node);
}

function walkTSAsExpression(node, visitors) {
  const enterExit = visitors[110];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSCallSignatureDeclaration(node, visitors) {
  const enterExit = visitors[111];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  if (exit !== null) exit(node);
}

function walkTSClassImplements(node, visitors) {
  const enterExit = visitors[112];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeArguments, visitors);
  if (exit !== null) exit(node);
}

function walkTSConditionalType(node, visitors) {
  const enterExit = visitors[113];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.checkType, visitors);
  walkNode(node.extendsType, visitors);
  walkNode(node.trueType, visitors);
  walkNode(node.falseType, visitors);
  if (exit !== null) exit(node);
}

function walkTSConstructSignatureDeclaration(node, visitors) {
  const enterExit = visitors[114];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  if (exit !== null) exit(node);
}

function walkTSConstructorType(node, visitors) {
  const enterExit = visitors[115];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  if (exit !== null) exit(node);
}

function walkTSDeclareFunction(node, visitors) {
  const enterExit = visitors[116];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkTSEmptyBodyFunctionExpression(node, visitors) {
  const enterExit = visitors[117];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  if (exit !== null) exit(node);
}

function walkTSEnumBody(node, visitors) {
  const enterExit = visitors[118];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.members, visitors);
  if (exit !== null) exit(node);
}

function walkTSEnumDeclaration(node, visitors) {
  const enterExit = visitors[119];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkTSEnumMember(node, visitors) {
  const enterExit = visitors[120];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.initializer, visitors);
  if (exit !== null) exit(node);
}

function walkTSExportAssignment(node, visitors) {
  const enterExit = visitors[121];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkTSExternalModuleReference(node, visitors) {
  const enterExit = visitors[122];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkTSFunctionType(node, visitors) {
  const enterExit = visitors[123];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  if (exit !== null) exit(node);
}

function walkTSImportEqualsDeclaration(node, visitors) {
  const enterExit = visitors[124];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.moduleReference, visitors);
  if (exit !== null) exit(node);
}

function walkTSImportType(node, visitors) {
  const enterExit = visitors[125];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.argument, visitors);
  walkNode(node.options, visitors);
  walkNode(node.qualifier, visitors);
  walkNode(node.typeArguments, visitors);
  if (exit !== null) exit(node);
}

function walkTSIndexSignature(node, visitors) {
  const enterExit = visitors[126];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.parameters, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSIndexedAccessType(node, visitors) {
  const enterExit = visitors[127];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.objectType, visitors);
  walkNode(node.indexType, visitors);
  if (exit !== null) exit(node);
}

function walkTSInferType(node, visitors) {
  const enterExit = visitors[128];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeParameter, visitors);
  if (exit !== null) exit(node);
}

function walkTSInstantiationExpression(node, visitors) {
  const enterExit = visitors[129];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeArguments, visitors);
  if (exit !== null) exit(node);
}

function walkTSInterfaceBody(node, visitors) {
  const enterExit = visitors[130];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkTSInterfaceDeclaration(node, visitors) {
  const enterExit = visitors[131];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.extends, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkTSInterfaceHeritage(node, visitors) {
  const enterExit = visitors[132];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeArguments, visitors);
  if (exit !== null) exit(node);
}

function walkTSIntersectionType(node, visitors) {
  const enterExit = visitors[133];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.types, visitors);
  if (exit !== null) exit(node);
}

function walkTSJSDocNonNullableType(node, visitors) {
  const enterExit = visitors[134];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSJSDocNullableType(node, visitors) {
  const enterExit = visitors[135];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSLiteralType(node, visitors) {
  const enterExit = visitors[136];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.literal, visitors);
  if (exit !== null) exit(node);
}

function walkTSMappedType(node, visitors) {
  const enterExit = visitors[137];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.constraint, visitors);
  walkNode(node.nameType, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSMethodSignature(node, visitors) {
  const enterExit = visitors[138];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  if (exit !== null) exit(node);
}

function walkTSModuleBlock(node, visitors) {
  const enterExit = visitors[139];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkTSModuleDeclaration(node, visitors) {
  const enterExit = visitors[140];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.body, visitors);
  if (exit !== null) exit(node);
}

function walkTSNamedTupleMember(node, visitors) {
  const enterExit = visitors[141];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.label, visitors);
  walkNode(node.elementType, visitors);
  if (exit !== null) exit(node);
}

function walkTSNamespaceExportDeclaration(node, visitors) {
  const enterExit = visitors[142];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  if (exit !== null) exit(node);
}

function walkTSNonNullExpression(node, visitors) {
  const enterExit = visitors[143];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkTSOptionalType(node, visitors) {
  const enterExit = visitors[144];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSParameterProperty(node, visitors) {
  const enterExit = visitors[145];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.parameter, visitors);
  if (exit !== null) exit(node);
}

function walkTSParenthesizedType(node, visitors) {
  const enterExit = visitors[146];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSPropertySignature(node, visitors) {
  const enterExit = visitors[147];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSQualifiedName(node, visitors) {
  const enterExit = visitors[148];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  if (exit !== null) exit(node);
}

function walkTSRestType(node, visitors) {
  const enterExit = visitors[149];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSSatisfiesExpression(node, visitors) {
  const enterExit = visitors[150];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSTemplateLiteralType(node, visitors) {
  const enterExit = visitors[151];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.quasis, visitors);
  walkNode(node.types, visitors);
  if (exit !== null) exit(node);
}

function walkTSTupleType(node, visitors) {
  const enterExit = visitors[152];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.elementTypes, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeAliasDeclaration(node, visitors) {
  const enterExit = visitors[153];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeAnnotation(node, visitors) {
  const enterExit = visitors[154];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeAssertion(node, visitors) {
  const enterExit = visitors[155];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  walkNode(node.expression, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeLiteral(node, visitors) {
  const enterExit = visitors[156];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.members, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeOperator(node, visitors) {
  const enterExit = visitors[157];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeParameter(node, visitors) {
  const enterExit = visitors[158];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.constraint, visitors);
  walkNode(node.default, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeParameterDeclaration(node, visitors) {
  const enterExit = visitors[159];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.params, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeParameterInstantiation(node, visitors) {
  const enterExit = visitors[160];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.params, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypePredicate(node, visitors) {
  const enterExit = visitors[161];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.parameterName, visitors);
  walkNode(node.typeAnnotation, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeQuery(node, visitors) {
  const enterExit = visitors[162];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.exprName, visitors);
  walkNode(node.typeArguments, visitors);
  if (exit !== null) exit(node);
}

function walkTSTypeReference(node, visitors) {
  const enterExit = visitors[163];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.typeName, visitors);
  walkNode(node.typeArguments, visitors);
  if (exit !== null) exit(node);
}

function walkTSUnionType(node, visitors) {
  const enterExit = visitors[164];
  let exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    if (enter !== null) enter(node);
  }
  walkNode(node.types, visitors);
  if (exit !== null) exit(node);
}
