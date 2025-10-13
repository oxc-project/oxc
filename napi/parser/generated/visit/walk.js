// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/estree_visit.rs`.

export { walkProgram };

const { isArray } = Array;

function walkNode(node, visitors) {
  if (node == null) return;
  if (isArray(node)) {
    let len = node.length;
    for (let i = 0; i < len; i++) walkNode(node[i], visitors);
  } else {switch (node.type) {
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
    }}
}

function walkDebuggerStatement(node, visitors) {
  let visit = visitors[0];
  visit !== null && visit(node);
}

function walkEmptyStatement(node, visitors) {
  let visit = visitors[1];
  visit !== null && visit(node);
}

function walkLiteral(node, visitors) {
  let visit = visitors[2];
  visit !== null && visit(node);
}

function walkPrivateIdentifier(node, visitors) {
  let visit = visitors[3];
  visit !== null && visit(node);
}

function walkSuper(node, visitors) {
  let visit = visitors[4];
  visit !== null && visit(node);
}

function walkTemplateElement(node, visitors) {
  let visit = visitors[5];
  visit !== null && visit(node);
}

function walkThisExpression(node, visitors) {
  let visit = visitors[6];
  visit !== null && visit(node);
}

function walkJSXClosingFragment(node, visitors) {
  let visit = visitors[7];
  visit !== null && visit(node);
}

function walkJSXEmptyExpression(node, visitors) {
  let visit = visitors[8];
  visit !== null && visit(node);
}

function walkJSXIdentifier(node, visitors) {
  let visit = visitors[9];
  visit !== null && visit(node);
}

function walkJSXOpeningFragment(node, visitors) {
  let visit = visitors[10];
  visit !== null && visit(node);
}

function walkJSXText(node, visitors) {
  let visit = visitors[11];
  visit !== null && visit(node);
}

function walkTSAnyKeyword(node, visitors) {
  let visit = visitors[12];
  visit !== null && visit(node);
}

function walkTSBigIntKeyword(node, visitors) {
  let visit = visitors[13];
  visit !== null && visit(node);
}

function walkTSBooleanKeyword(node, visitors) {
  let visit = visitors[14];
  visit !== null && visit(node);
}

function walkTSIntrinsicKeyword(node, visitors) {
  let visit = visitors[15];
  visit !== null && visit(node);
}

function walkTSJSDocUnknownType(node, visitors) {
  let visit = visitors[16];
  visit !== null && visit(node);
}

function walkTSNeverKeyword(node, visitors) {
  let visit = visitors[17];
  visit !== null && visit(node);
}

function walkTSNullKeyword(node, visitors) {
  let visit = visitors[18];
  visit !== null && visit(node);
}

function walkTSNumberKeyword(node, visitors) {
  let visit = visitors[19];
  visit !== null && visit(node);
}

function walkTSObjectKeyword(node, visitors) {
  let visit = visitors[20];
  visit !== null && visit(node);
}

function walkTSStringKeyword(node, visitors) {
  let visit = visitors[21];
  visit !== null && visit(node);
}

function walkTSSymbolKeyword(node, visitors) {
  let visit = visitors[22];
  visit !== null && visit(node);
}

function walkTSThisType(node, visitors) {
  let visit = visitors[23];
  visit !== null && visit(node);
}

function walkTSUndefinedKeyword(node, visitors) {
  let visit = visitors[24];
  visit !== null && visit(node);
}

function walkTSUnknownKeyword(node, visitors) {
  let visit = visitors[25];
  visit !== null && visit(node);
}

function walkTSVoidKeyword(node, visitors) {
  let visit = visitors[26];
  visit !== null && visit(node);
}

function walkAccessorProperty(node, visitors) {
  let enterExit = visitors[27], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkArrayExpression(node, visitors) {
  let enterExit = visitors[28], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.elements, visitors);
  exit !== null && exit(node);
}

function walkArrayPattern(node, visitors) {
  let enterExit = visitors[29], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.elements, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkArrowFunctionExpression(node, visitors) {
  let enterExit = visitors[30], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkAssignmentExpression(node, visitors) {
  let enterExit = visitors[31], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  exit !== null && exit(node);
}

function walkAssignmentPattern(node, visitors) {
  let enterExit = visitors[32], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkAwaitExpression(node, visitors) {
  let enterExit = visitors[33], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkBinaryExpression(node, visitors) {
  let enterExit = visitors[34], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  exit !== null && exit(node);
}

function walkBlockStatement(node, visitors) {
  let enterExit = visitors[35], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkBreakStatement(node, visitors) {
  let enterExit = visitors[36], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.label, visitors);
  exit !== null && exit(node);
}

function walkCallExpression(node, visitors) {
  let enterExit = visitors[37], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.callee, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.arguments, visitors);
  exit !== null && exit(node);
}

function walkCatchClause(node, visitors) {
  let enterExit = visitors[38], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.param, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkChainExpression(node, visitors) {
  let enterExit = visitors[39], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkClassBody(node, visitors) {
  let enterExit = visitors[40], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkClassDeclaration(node, visitors) {
  let enterExit = visitors[41], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.superClass, visitors);
  walkNode(node.superTypeArguments, visitors);
  walkNode(node.implements, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkClassExpression(node, visitors) {
  let enterExit = visitors[42], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.superClass, visitors);
  walkNode(node.superTypeArguments, visitors);
  walkNode(node.implements, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkConditionalExpression(node, visitors) {
  let enterExit = visitors[43], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.consequent, visitors);
  walkNode(node.alternate, visitors);
  exit !== null && exit(node);
}

function walkContinueStatement(node, visitors) {
  let enterExit = visitors[44], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.label, visitors);
  exit !== null && exit(node);
}

function walkDecorator(node, visitors) {
  let enterExit = visitors[45], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkDoWhileStatement(node, visitors) {
  let enterExit = visitors[46], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  walkNode(node.test, visitors);
  exit !== null && exit(node);
}

function walkExportAllDeclaration(node, visitors) {
  let enterExit = visitors[47], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.exported, visitors);
  walkNode(node.source, visitors);
  walkNode(node.attributes, visitors);
  exit !== null && exit(node);
}

function walkExportDefaultDeclaration(node, visitors) {
  let enterExit = visitors[48], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.declaration, visitors);
  exit !== null && exit(node);
}

function walkExportNamedDeclaration(node, visitors) {
  let enterExit = visitors[49], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.declaration, visitors);
  walkNode(node.specifiers, visitors);
  walkNode(node.source, visitors);
  walkNode(node.attributes, visitors);
  exit !== null && exit(node);
}

function walkExportSpecifier(node, visitors) {
  let enterExit = visitors[50], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.local, visitors);
  walkNode(node.exported, visitors);
  exit !== null && exit(node);
}

function walkExpressionStatement(node, visitors) {
  let enterExit = visitors[51], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkForInStatement(node, visitors) {
  let enterExit = visitors[52], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkForOfStatement(node, visitors) {
  let enterExit = visitors[53], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkForStatement(node, visitors) {
  let enterExit = visitors[54], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.init, visitors);
  walkNode(node.test, visitors);
  walkNode(node.update, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkFunctionDeclaration(node, visitors) {
  let enterExit = visitors[55], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkFunctionExpression(node, visitors) {
  let enterExit = visitors[56], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkIdentifier(node, visitors) {
  let enterExit = visitors[57], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkIfStatement(node, visitors) {
  let enterExit = visitors[58], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.consequent, visitors);
  walkNode(node.alternate, visitors);
  exit !== null && exit(node);
}

function walkImportAttribute(node, visitors) {
  let enterExit = visitors[59], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkImportDeclaration(node, visitors) {
  let enterExit = visitors[60], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.specifiers, visitors);
  walkNode(node.source, visitors);
  walkNode(node.attributes, visitors);
  exit !== null && exit(node);
}

function walkImportDefaultSpecifier(node, visitors) {
  let enterExit = visitors[61], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.local, visitors);
  exit !== null && exit(node);
}

function walkImportExpression(node, visitors) {
  let enterExit = visitors[62], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.source, visitors);
  walkNode(node.options, visitors);
  exit !== null && exit(node);
}

function walkImportNamespaceSpecifier(node, visitors) {
  let enterExit = visitors[63], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.local, visitors);
  exit !== null && exit(node);
}

function walkImportSpecifier(node, visitors) {
  let enterExit = visitors[64], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.imported, visitors);
  walkNode(node.local, visitors);
  exit !== null && exit(node);
}

function walkLabeledStatement(node, visitors) {
  let enterExit = visitors[65], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.label, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkLogicalExpression(node, visitors) {
  let enterExit = visitors[66], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  exit !== null && exit(node);
}

function walkMemberExpression(node, visitors) {
  let enterExit = visitors[67], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.object, visitors);
  walkNode(node.property, visitors);
  exit !== null && exit(node);
}

function walkMetaProperty(node, visitors) {
  let enterExit = visitors[68], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.meta, visitors);
  walkNode(node.property, visitors);
  exit !== null && exit(node);
}

function walkMethodDefinition(node, visitors) {
  let enterExit = visitors[69], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkNewExpression(node, visitors) {
  let enterExit = visitors[70], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.callee, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.arguments, visitors);
  exit !== null && exit(node);
}

function walkObjectExpression(node, visitors) {
  let enterExit = visitors[71], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.properties, visitors);
  exit !== null && exit(node);
}

function walkObjectPattern(node, visitors) {
  let enterExit = visitors[72], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.properties, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkParenthesizedExpression(node, visitors) {
  let enterExit = visitors[73], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkProgram(node, visitors) {
  let enterExit = visitors[74], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkProperty(node, visitors) {
  let enterExit = visitors[75], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkPropertyDefinition(node, visitors) {
  let enterExit = visitors[76], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkRestElement(node, visitors) {
  let enterExit = visitors[77], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.argument, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkReturnStatement(node, visitors) {
  let enterExit = visitors[78], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkSequenceExpression(node, visitors) {
  let enterExit = visitors[79], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expressions, visitors);
  exit !== null && exit(node);
}

function walkSpreadElement(node, visitors) {
  let enterExit = visitors[80], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkStaticBlock(node, visitors) {
  let enterExit = visitors[81], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkSwitchCase(node, visitors) {
  let enterExit = visitors[82], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.consequent, visitors);
  exit !== null && exit(node);
}

function walkSwitchStatement(node, visitors) {
  let enterExit = visitors[83], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.discriminant, visitors);
  walkNode(node.cases, visitors);
  exit !== null && exit(node);
}

function walkTaggedTemplateExpression(node, visitors) {
  let enterExit = visitors[84], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.tag, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.quasi, visitors);
  exit !== null && exit(node);
}

function walkTemplateLiteral(node, visitors) {
  let enterExit = visitors[85], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.quasis, visitors);
  walkNode(node.expressions, visitors);
  exit !== null && exit(node);
}

function walkThrowStatement(node, visitors) {
  let enterExit = visitors[86], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkTryStatement(node, visitors) {
  let enterExit = visitors[87], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.block, visitors);
  walkNode(node.handler, visitors);
  walkNode(node.finalizer, visitors);
  exit !== null && exit(node);
}

function walkUnaryExpression(node, visitors) {
  let enterExit = visitors[88], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkUpdateExpression(node, visitors) {
  let enterExit = visitors[89], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkV8IntrinsicExpression(node, visitors) {
  let enterExit = visitors[90], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.arguments, visitors);
  exit !== null && exit(node);
}

function walkVariableDeclaration(node, visitors) {
  let enterExit = visitors[91], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.declarations, visitors);
  exit !== null && exit(node);
}

function walkVariableDeclarator(node, visitors) {
  let enterExit = visitors[92], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.init, visitors);
  exit !== null && exit(node);
}

function walkWhileStatement(node, visitors) {
  let enterExit = visitors[93], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.test, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkWithStatement(node, visitors) {
  let enterExit = visitors[94], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.object, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkYieldExpression(node, visitors) {
  let enterExit = visitors[95], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkJSXAttribute(node, visitors) {
  let enterExit = visitors[96], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkJSXClosingElement(node, visitors) {
  let enterExit = visitors[97], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.name, visitors);
  exit !== null && exit(node);
}

function walkJSXElement(node, visitors) {
  let enterExit = visitors[98], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.openingElement, visitors);
  walkNode(node.children, visitors);
  walkNode(node.closingElement, visitors);
  exit !== null && exit(node);
}

function walkJSXExpressionContainer(node, visitors) {
  let enterExit = visitors[99], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkJSXFragment(node, visitors) {
  let enterExit = visitors[100], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.openingFragment, visitors);
  walkNode(node.children, visitors);
  walkNode(node.closingFragment, visitors);
  exit !== null && exit(node);
}

function walkJSXMemberExpression(node, visitors) {
  let enterExit = visitors[101], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.object, visitors);
  walkNode(node.property, visitors);
  exit !== null && exit(node);
}

function walkJSXNamespacedName(node, visitors) {
  let enterExit = visitors[102], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.namespace, visitors);
  walkNode(node.name, visitors);
  exit !== null && exit(node);
}

function walkJSXOpeningElement(node, visitors) {
  let enterExit = visitors[103], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.typeArguments, visitors);
  walkNode(node.attributes, visitors);
  exit !== null && exit(node);
}

function walkJSXSpreadAttribute(node, visitors) {
  let enterExit = visitors[104], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  exit !== null && exit(node);
}

function walkJSXSpreadChild(node, visitors) {
  let enterExit = visitors[105], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkTSAbstractAccessorProperty(node, visitors) {
  let enterExit = visitors[106], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSAbstractMethodDefinition(node, visitors) {
  let enterExit = visitors[107], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.value, visitors);
  exit !== null && exit(node);
}

function walkTSAbstractPropertyDefinition(node, visitors) {
  let enterExit = visitors[108], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSArrayType(node, visitors) {
  let enterExit = visitors[109], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.elementType, visitors);
  exit !== null && exit(node);
}

function walkTSAsExpression(node, visitors) {
  let enterExit = visitors[110], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSCallSignatureDeclaration(node, visitors) {
  let enterExit = visitors[111], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  exit !== null && exit(node);
}

function walkTSClassImplements(node, visitors) {
  let enterExit = visitors[112], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeArguments, visitors);
  exit !== null && exit(node);
}

function walkTSConditionalType(node, visitors) {
  let enterExit = visitors[113], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.checkType, visitors);
  walkNode(node.extendsType, visitors);
  walkNode(node.trueType, visitors);
  walkNode(node.falseType, visitors);
  exit !== null && exit(node);
}

function walkTSConstructSignatureDeclaration(node, visitors) {
  let enterExit = visitors[114], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  exit !== null && exit(node);
}

function walkTSConstructorType(node, visitors) {
  let enterExit = visitors[115], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  exit !== null && exit(node);
}

function walkTSDeclareFunction(node, visitors) {
  let enterExit = visitors[116], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkTSEmptyBodyFunctionExpression(node, visitors) {
  let enterExit = visitors[117], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  exit !== null && exit(node);
}

function walkTSEnumBody(node, visitors) {
  let enterExit = visitors[118], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.members, visitors);
  exit !== null && exit(node);
}

function walkTSEnumDeclaration(node, visitors) {
  let enterExit = visitors[119], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkTSEnumMember(node, visitors) {
  let enterExit = visitors[120], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.initializer, visitors);
  exit !== null && exit(node);
}

function walkTSExportAssignment(node, visitors) {
  let enterExit = visitors[121], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkTSExternalModuleReference(node, visitors) {
  let enterExit = visitors[122], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkTSFunctionType(node, visitors) {
  let enterExit = visitors[123], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  exit !== null && exit(node);
}

function walkTSImportEqualsDeclaration(node, visitors) {
  let enterExit = visitors[124], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.moduleReference, visitors);
  exit !== null && exit(node);
}

function walkTSImportType(node, visitors) {
  let enterExit = visitors[125], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.argument, visitors);
  walkNode(node.options, visitors);
  walkNode(node.qualifier, visitors);
  walkNode(node.typeArguments, visitors);
  exit !== null && exit(node);
}

function walkTSIndexSignature(node, visitors) {
  let enterExit = visitors[126], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.parameters, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSIndexedAccessType(node, visitors) {
  let enterExit = visitors[127], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.objectType, visitors);
  walkNode(node.indexType, visitors);
  exit !== null && exit(node);
}

function walkTSInferType(node, visitors) {
  let enterExit = visitors[128], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeParameter, visitors);
  exit !== null && exit(node);
}

function walkTSInstantiationExpression(node, visitors) {
  let enterExit = visitors[129], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeArguments, visitors);
  exit !== null && exit(node);
}

function walkTSInterfaceBody(node, visitors) {
  let enterExit = visitors[130], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkTSInterfaceDeclaration(node, visitors) {
  let enterExit = visitors[131], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.extends, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkTSInterfaceHeritage(node, visitors) {
  let enterExit = visitors[132], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeArguments, visitors);
  exit !== null && exit(node);
}

function walkTSIntersectionType(node, visitors) {
  let enterExit = visitors[133], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.types, visitors);
  exit !== null && exit(node);
}

function walkTSJSDocNonNullableType(node, visitors) {
  let enterExit = visitors[134], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSJSDocNullableType(node, visitors) {
  let enterExit = visitors[135], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSLiteralType(node, visitors) {
  let enterExit = visitors[136], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.literal, visitors);
  exit !== null && exit(node);
}

function walkTSMappedType(node, visitors) {
  let enterExit = visitors[137], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.constraint, visitors);
  walkNode(node.nameType, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSMethodSignature(node, visitors) {
  let enterExit = visitors[138], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.params, visitors);
  walkNode(node.returnType, visitors);
  exit !== null && exit(node);
}

function walkTSModuleBlock(node, visitors) {
  let enterExit = visitors[139], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkTSModuleDeclaration(node, visitors) {
  let enterExit = visitors[140], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.body, visitors);
  exit !== null && exit(node);
}

function walkTSNamedTupleMember(node, visitors) {
  let enterExit = visitors[141], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.label, visitors);
  walkNode(node.elementType, visitors);
  exit !== null && exit(node);
}

function walkTSNamespaceExportDeclaration(node, visitors) {
  let enterExit = visitors[142], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  exit !== null && exit(node);
}

function walkTSNonNullExpression(node, visitors) {
  let enterExit = visitors[143], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkTSOptionalType(node, visitors) {
  let enterExit = visitors[144], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSParameterProperty(node, visitors) {
  let enterExit = visitors[145], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.decorators, visitors);
  walkNode(node.parameter, visitors);
  exit !== null && exit(node);
}

function walkTSParenthesizedType(node, visitors) {
  let enterExit = visitors[146], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSPropertySignature(node, visitors) {
  let enterExit = visitors[147], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.key, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSQualifiedName(node, visitors) {
  let enterExit = visitors[148], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.left, visitors);
  walkNode(node.right, visitors);
  exit !== null && exit(node);
}

function walkTSRestType(node, visitors) {
  let enterExit = visitors[149], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSSatisfiesExpression(node, visitors) {
  let enterExit = visitors[150], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.expression, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSTemplateLiteralType(node, visitors) {
  let enterExit = visitors[151], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.quasis, visitors);
  walkNode(node.types, visitors);
  exit !== null && exit(node);
}

function walkTSTupleType(node, visitors) {
  let enterExit = visitors[152], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.elementTypes, visitors);
  exit !== null && exit(node);
}

function walkTSTypeAliasDeclaration(node, visitors) {
  let enterExit = visitors[153], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.id, visitors);
  walkNode(node.typeParameters, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSTypeAnnotation(node, visitors) {
  let enterExit = visitors[154], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSTypeAssertion(node, visitors) {
  let enterExit = visitors[155], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  walkNode(node.expression, visitors);
  exit !== null && exit(node);
}

function walkTSTypeLiteral(node, visitors) {
  let enterExit = visitors[156], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.members, visitors);
  exit !== null && exit(node);
}

function walkTSTypeOperator(node, visitors) {
  let enterExit = visitors[157], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSTypeParameter(node, visitors) {
  let enterExit = visitors[158], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.name, visitors);
  walkNode(node.constraint, visitors);
  walkNode(node.default, visitors);
  exit !== null && exit(node);
}

function walkTSTypeParameterDeclaration(node, visitors) {
  let enterExit = visitors[159], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.params, visitors);
  exit !== null && exit(node);
}

function walkTSTypeParameterInstantiation(node, visitors) {
  let enterExit = visitors[160], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.params, visitors);
  exit !== null && exit(node);
}

function walkTSTypePredicate(node, visitors) {
  let enterExit = visitors[161], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.parameterName, visitors);
  walkNode(node.typeAnnotation, visitors);
  exit !== null && exit(node);
}

function walkTSTypeQuery(node, visitors) {
  let enterExit = visitors[162], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.exprName, visitors);
  walkNode(node.typeArguments, visitors);
  exit !== null && exit(node);
}

function walkTSTypeReference(node, visitors) {
  let enterExit = visitors[163], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.typeName, visitors);
  walkNode(node.typeArguments, visitors);
  exit !== null && exit(node);
}

function walkTSUnionType(node, visitors) {
  let enterExit = visitors[164], exit = null;
  if (enterExit !== null) {
    let enter;
    ({ enter, exit } = enterExit);
    enter !== null && enter(node);
  }
  walkNode(node.types, visitors);
  exit !== null && exit(node);
}
