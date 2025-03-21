// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`

'use strict';

module.exports = deserialize;

let uint8, uint32, float64, sourceText, sourceIsAscii, sourceLen;

const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true }),
  decodeStr = textDecoder.decode.bind(textDecoder),
  { fromCodePoint } = String;

function deserialize(buffer, sourceTextInput, sourceLenInput) {
  uint8 = buffer;
  uint32 = new Uint32Array(buffer.buffer, buffer.byteOffset);
  float64 = new Float64Array(buffer.buffer, buffer.byteOffset);

  sourceText = sourceTextInput;
  sourceLen = sourceLenInput;
  sourceIsAscii = sourceText.length === sourceLen;

  // (2 * 1024 * 1024 * 1024 - 16) >> 2
  const metadataPos32 = 536870908;

  const data = deserializeRawTransferData(uint32[metadataPos32]);

  uint8 =
    uint32 =
    float64 =
    sourceText =
      undefined;

  return data;
}

function deserializeProgram(pos) {
  const body = deserializeVecDirective(pos + 88);
  body.push(...deserializeVecStatement(pos + 120));
  return {
    type: 'Program',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body,
    sourceType: deserializeModuleKind(pos + 9),
    hashbang: deserializeOptionHashbang(pos + 64),
  };
}

function deserializeIdentifierName(pos) {
  return {
    type: 'Identifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
    decorators: [],
    optional: false,
    typeAnnotation: null,
  };
}

function deserializeIdentifierReference(pos) {
  return {
    type: 'Identifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
    decorators: [],
    optional: false,
    typeAnnotation: null,
  };
}

function deserializeBindingIdentifier(pos) {
  return {
    type: 'Identifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
    decorators: [],
    optional: false,
    typeAnnotation: null,
  };
}

function deserializeLabelIdentifier(pos) {
  return {
    type: 'Identifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
  };
}

function deserializeThisExpression(pos) {
  return {
    type: 'ThisExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeArrayExpression(pos) {
  return {
    type: 'ArrayExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    elements: deserializeVecArrayExpressionElement(pos + 8),
  };
}

function deserializeElision(pos) {
  return null;
}

function deserializeObjectExpression(pos) {
  return {
    type: 'ObjectExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    properties: deserializeVecObjectPropertyKind(pos + 8),
  };
}

function deserializeObjectProperty(pos) {
  return {
    type: 'Property',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    method: deserializeBool(pos + 48),
    shorthand: deserializeBool(pos + 49),
    computed: deserializeBool(pos + 50),
    key: deserializePropertyKey(pos + 16),
    value: deserializeExpression(pos + 32),
    kind: deserializePropertyKind(pos + 8),
  };
}

function deserializeTemplateLiteral(pos) {
  return {
    type: 'TemplateLiteral',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expressions: deserializeVecExpression(pos + 40),
    quasis: deserializeVecTemplateElement(pos + 8),
  };
}

function deserializeTaggedTemplateExpression(pos) {
  return {
    type: 'TaggedTemplateExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    tag: deserializeExpression(pos + 8),
    quasi: deserializeTemplateLiteral(pos + 24),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 96),
  };
}

function deserializeTemplateElement(pos) {
  return {
    type: 'TemplateElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value: deserializeTemplateElementValue(pos + 8),
    tail: deserializeBool(pos + 40),
  };
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  return {
    type: 'MemberExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    object: deserializeExpression(pos + 8),
    property: deserializeExpression(pos + 24),
    computed: true,
    optional: deserializeBool(pos + 40),
  };
}

function deserializeStaticMemberExpression(pos) {
  return {
    type: 'MemberExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    object: deserializeExpression(pos + 8),
    property: deserializeIdentifierName(pos + 24),
    computed: false,
    optional: deserializeBool(pos + 48),
  };
}

function deserializePrivateFieldExpression(pos) {
  return {
    type: 'MemberExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    object: deserializeExpression(pos + 8),
    property: deserializePrivateIdentifier(pos + 24),
    computed: false,
    optional: deserializeBool(pos + 48),
  };
}

function deserializeCallExpression(pos) {
  return {
    type: 'CallExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    callee: deserializeExpression(pos + 8),
    arguments: deserializeVecArgument(pos + 32),
    optional: deserializeBool(pos + 64),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
  };
}

function deserializeNewExpression(pos) {
  return {
    type: 'NewExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    callee: deserializeExpression(pos + 8),
    arguments: deserializeVecArgument(pos + 24),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 56),
  };
}

function deserializeMetaProperty(pos) {
  return {
    type: 'MetaProperty',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    meta: deserializeIdentifierName(pos + 8),
    property: deserializeIdentifierName(pos + 32),
  };
}

function deserializeSpreadElement(pos) {
  return {
    type: 'SpreadElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeExpression(pos + 8),
  };
}

function deserializeUpdateExpression(pos) {
  return {
    type: 'UpdateExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    operator: deserializeUpdateOperator(pos + 8),
    prefix: deserializeBool(pos + 9),
    argument: deserializeSimpleAssignmentTarget(pos + 16),
  };
}

function deserializeUnaryExpression(pos) {
  return {
    type: 'UnaryExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    operator: deserializeUnaryOperator(pos + 8),
    prefix: true,
    argument: deserializeExpression(pos + 16),
  };
}

function deserializeBinaryExpression(pos) {
  return {
    type: 'BinaryExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializeExpression(pos + 8),
    operator: deserializeBinaryOperator(pos + 24),
    right: deserializeExpression(pos + 32),
  };
}

function deserializePrivateInExpression(pos) {
  return {
    type: 'BinaryExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializePrivateIdentifier(pos + 8),
    operator: 'in',
    right: deserializeExpression(pos + 32),
  };
}

function deserializeLogicalExpression(pos) {
  return {
    type: 'LogicalExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializeExpression(pos + 8),
    operator: deserializeLogicalOperator(pos + 24),
    right: deserializeExpression(pos + 32),
  };
}

function deserializeConditionalExpression(pos) {
  return {
    type: 'ConditionalExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    test: deserializeExpression(pos + 8),
    consequent: deserializeExpression(pos + 24),
    alternate: deserializeExpression(pos + 40),
  };
}

function deserializeAssignmentExpression(pos) {
  return {
    type: 'AssignmentExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    operator: deserializeAssignmentOperator(pos + 8),
    left: deserializeAssignmentTarget(pos + 16),
    right: deserializeExpression(pos + 32),
  };
}

function deserializeArrayAssignmentTarget(pos) {
  const elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 8);
  const rest = deserializeOptionAssignmentTargetRest(pos + 40);
  if (rest !== null) elements.push(rest);
  return {
    type: 'ArrayPattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    elements,
  };
}

function deserializeObjectAssignmentTarget(pos) {
  const properties = deserializeVecAssignmentTargetProperty(pos + 8);
  const rest = deserializeOptionAssignmentTargetRest(pos + 40);
  if (rest !== null) properties.push(rest);
  return {
    type: 'ObjectPattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    properties,
  };
}

function deserializeAssignmentTargetRest(pos) {
  return {
    type: 'RestElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeAssignmentTarget(pos + 8),
  };
}

function deserializeAssignmentTargetWithDefault(pos) {
  return {
    type: 'AssignmentPattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializeAssignmentTarget(pos + 8),
    right: deserializeExpression(pos + 24),
  };
}

function deserializeAssignmentTargetPropertyIdentifier(pos) {
  const start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    key = deserializeIdentifierReference(pos + 8);
  const init = deserializeOptionExpression(pos + 40),
    keyCopy = { ...key },
    value = init === null
      ? keyCopy
      : {
        type: 'AssignmentPattern',
        start: start,
        end: end,
        left: keyCopy,
        right: init,
      };
  return {
    type: 'Property',
    start,
    end,
    method: false,
    shorthand: true,
    computed: false,
    key,
    value,
    kind: 'init',
  };
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  return {
    type: 'Property',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    method: false,
    shorthand: false,
    computed: deserializeBool(pos + 40),
    key: deserializePropertyKey(pos + 8),
    value: deserializeAssignmentTargetMaybeDefault(pos + 24),
    kind: 'init',
  };
}

function deserializeSequenceExpression(pos) {
  return {
    type: 'SequenceExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expressions: deserializeVecExpression(pos + 8),
  };
}

function deserializeSuper(pos) {
  return {
    type: 'Super',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeAwaitExpression(pos) {
  return {
    type: 'AwaitExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeExpression(pos + 8),
  };
}

function deserializeChainExpression(pos) {
  return {
    type: 'ChainExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeChainElement(pos + 8),
  };
}

function deserializeParenthesizedExpression(pos) {
  return {
    type: 'ParenthesizedExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
  };
}

function deserializeDirective(pos) {
  return {
    type: 'ExpressionStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeStringLiteral(pos + 8),
    directive: deserializeStr(pos + 56),
  };
}

function deserializeHashbang(pos) {
  return {
    type: 'Hashbang',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value: deserializeStr(pos + 8),
  };
}

function deserializeBlockStatement(pos) {
  return {
    type: 'BlockStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeVecStatement(pos + 8),
  };
}

function deserializeVariableDeclaration(pos) {
  return {
    type: 'VariableDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    declarations: deserializeVecVariableDeclarator(pos + 16),
    kind: deserializeVariableDeclarationKind(pos + 8),
    declare: deserializeBool(pos + 48),
  };
}

function deserializeVariableDeclarator(pos) {
  return {
    type: 'VariableDeclarator',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeBindingPattern(pos + 16),
    init: deserializeOptionExpression(pos + 48),
    definite: deserializeBool(pos + 64),
  };
}

function deserializeEmptyStatement(pos) {
  return {
    type: 'EmptyStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeExpressionStatement(pos) {
  return {
    type: 'ExpressionStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
    directive: null,
  };
}

function deserializeIfStatement(pos) {
  return {
    type: 'IfStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    test: deserializeExpression(pos + 8),
    consequent: deserializeStatement(pos + 24),
    alternate: deserializeOptionStatement(pos + 40),
  };
}

function deserializeDoWhileStatement(pos) {
  return {
    type: 'DoWhileStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeStatement(pos + 8),
    test: deserializeExpression(pos + 24),
  };
}

function deserializeWhileStatement(pos) {
  return {
    type: 'WhileStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    test: deserializeExpression(pos + 8),
    body: deserializeStatement(pos + 24),
  };
}

function deserializeForStatement(pos) {
  return {
    type: 'ForStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    init: deserializeOptionForStatementInit(pos + 8),
    test: deserializeOptionExpression(pos + 24),
    update: deserializeOptionExpression(pos + 40),
    body: deserializeStatement(pos + 56),
  };
}

function deserializeForInStatement(pos) {
  return {
    type: 'ForInStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializeForStatementLeft(pos + 8),
    right: deserializeExpression(pos + 24),
    body: deserializeStatement(pos + 40),
  };
}

function deserializeForOfStatement(pos) {
  return {
    type: 'ForOfStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    await: deserializeBool(pos + 8),
    left: deserializeForStatementLeft(pos + 16),
    right: deserializeExpression(pos + 32),
    body: deserializeStatement(pos + 48),
  };
}

function deserializeContinueStatement(pos) {
  return {
    type: 'ContinueStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    label: deserializeOptionLabelIdentifier(pos + 8),
  };
}

function deserializeBreakStatement(pos) {
  return {
    type: 'BreakStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    label: deserializeOptionLabelIdentifier(pos + 8),
  };
}

function deserializeReturnStatement(pos) {
  return {
    type: 'ReturnStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeOptionExpression(pos + 8),
  };
}

function deserializeWithStatement(pos) {
  return {
    type: 'WithStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    object: deserializeExpression(pos + 8),
    body: deserializeStatement(pos + 24),
  };
}

function deserializeSwitchStatement(pos) {
  return {
    type: 'SwitchStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    discriminant: deserializeExpression(pos + 8),
    cases: deserializeVecSwitchCase(pos + 24),
  };
}

function deserializeSwitchCase(pos) {
  return {
    type: 'SwitchCase',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    consequent: deserializeVecStatement(pos + 24),
    test: deserializeOptionExpression(pos + 8),
  };
}

function deserializeLabeledStatement(pos) {
  return {
    type: 'LabeledStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeStatement(pos + 32),
    label: deserializeLabelIdentifier(pos + 8),
  };
}

function deserializeThrowStatement(pos) {
  return {
    type: 'ThrowStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeExpression(pos + 8),
  };
}

function deserializeTryStatement(pos) {
  return {
    type: 'TryStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    block: deserializeBoxBlockStatement(pos + 8),
    handler: deserializeOptionBoxCatchClause(pos + 16),
    finalizer: deserializeOptionBoxBlockStatement(pos + 24),
  };
}

function deserializeCatchClause(pos) {
  return {
    type: 'CatchClause',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    param: deserializeOptionCatchParameter(pos + 8),
    body: deserializeBoxBlockStatement(pos + 48),
  };
}

function deserializeCatchParameter(pos) {
  return {
    ...deserializeBindingPatternKind(pos + 8),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 24),
    optional: deserializeBool(pos + 32),
  };
}

function deserializeDebuggerStatement(pos) {
  return {
    type: 'DebuggerStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeBindingPattern(pos) {
  return {
    ...deserializeBindingPatternKind(pos),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 16),
    optional: deserializeBool(pos + 24),
  };
}

function deserializeAssignmentPattern(pos) {
  return {
    type: 'AssignmentPattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializeBindingPattern(pos + 8),
    right: deserializeExpression(pos + 40),
  };
}

function deserializeObjectPattern(pos) {
  const properties = deserializeVecBindingProperty(pos + 8);
  const rest = deserializeOptionBoxBindingRestElement(pos + 40);
  if (rest !== null) properties.push(rest);
  return {
    type: 'ObjectPattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    properties,
  };
}

function deserializeBindingProperty(pos) {
  return {
    type: 'Property',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    method: false,
    shorthand: deserializeBool(pos + 56),
    computed: deserializeBool(pos + 57),
    key: deserializePropertyKey(pos + 8),
    value: deserializeBindingPattern(pos + 24),
    kind: 'init',
  };
}

function deserializeArrayPattern(pos) {
  const elements = deserializeVecOptionBindingPattern(pos + 8);
  const rest = deserializeOptionBoxBindingRestElement(pos + 40);
  if (rest !== null) elements.push(rest);
  return {
    type: 'ArrayPattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    elements,
  };
}

function deserializeBindingRestElement(pos) {
  return {
    type: 'RestElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeBindingPattern(pos + 8),
  };
}

function deserializeFunction(pos) {
  const params = deserializeBoxFormalParameters(pos + 72);
  const thisParam = deserializeOptionBoxTSThisParameter(pos + 64);
  if (thisParam !== null) params.unshift(thisParam);
  return {
    type: deserializeFunctionType(pos + 8),
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeOptionBindingIdentifier(pos + 16),
    expression: false,
    generator: deserializeBool(pos + 48),
    async: deserializeBool(pos + 49),
    params,
    body: deserializeOptionBoxFunctionBody(pos + 88),
    declare: deserializeBool(pos + 50),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 56),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 80),
  };
}

function deserializeFormalParameters(pos) {
  const params = deserializeVecFormalParameter(pos + 16);
  if (uint32[(pos + 48) >> 2] !== 0 && uint32[(pos + 52) >> 2] !== 0) {
    pos = uint32[(pos + 48) >> 2];
    params.push({
      type: 'RestElement',
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
      argument: deserializeBindingPatternKind(pos + 8),
      typeAnnotation: deserializeOptionBoxTSTypeAnnotation(
        pos + 24,
      ),
      optional: deserializeBool(pos + 32),
    });
  }
  return params;
}

function deserializeFormalParameter(pos) {
  return {
    ...deserializeBindingPatternKind(pos + 40),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 56),
    optional: deserializeBool(pos + 64),
    decorators: deserializeVecDecorator(pos + 8),
  };
}

function deserializeFunctionBody(pos) {
  const body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 40));
  return {
    type: 'BlockStatement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body,
  };
}

function deserializeArrowFunctionExpression(pos) {
  const expression = deserializeBool(pos + 8);
  let body = deserializeBoxFunctionBody(pos + 40);
  return {
    type: 'ArrowFunctionExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: null,
    expression,
    generator: false,
    async: deserializeBool(pos + 9),
    params: deserializeBoxFormalParameters(pos + 24),
    body: expression ? body.body[0].expression : body,
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 16),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 32),
  };
}

function deserializeYieldExpression(pos) {
  return {
    type: 'YieldExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    delegate: deserializeBool(pos + 8),
    argument: deserializeOptionExpression(pos + 16),
  };
}

function deserializeClass(pos) {
  const classImplements = deserializeOptionVecTSClassImplements(pos + 112);
  return {
    type: deserializeClassType(pos + 8),
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeOptionBindingIdentifier(pos + 48),
    superClass: deserializeOptionExpression(pos + 88),
    body: deserializeBoxClassBody(pos + 144),
    decorators: deserializeVecDecorator(pos + 16),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 80),
    superTypeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 104),
    implements: classImplements === null ? [] : classImplements,
    abstract: deserializeBool(pos + 152),
    declare: deserializeBool(pos + 153),
  };
}

function deserializeClassBody(pos) {
  return {
    type: 'ClassBody',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeVecClassElement(pos + 8),
  };
}

function deserializeMethodDefinition(pos) {
  return {
    type: deserializeMethodDefinitionType(pos + 8),
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    static: deserializeBool(pos + 74),
    computed: deserializeBool(pos + 73),
    key: deserializePropertyKey(pos + 48),
    kind: deserializeMethodDefinitionKind(pos + 72),
    value: deserializeBoxFunction(pos + 64),
    decorators: deserializeVecDecorator(pos + 16),
    override: deserializeBool(pos + 75),
    optional: deserializeBool(pos + 76),
    accessibility: deserializeOptionTSAccessibility(pos + 77),
  };
}

function deserializePropertyDefinition(pos) {
  return {
    type: deserializePropertyDefinitionType(pos + 8),
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    static: deserializeBool(pos + 81),
    computed: deserializeBool(pos + 80),
    key: deserializePropertyKey(pos + 48),
    value: deserializeOptionExpression(pos + 64),
    decorators: deserializeVecDecorator(pos + 16),
    declare: deserializeBool(pos + 82),
    override: deserializeBool(pos + 83),
    optional: deserializeBool(pos + 84),
    definite: deserializeBool(pos + 85),
    readonly: deserializeBool(pos + 86),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 88),
    accessibility: deserializeOptionTSAccessibility(pos + 96),
  };
}

function deserializePrivateIdentifier(pos) {
  return {
    type: 'PrivateIdentifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
  };
}

function deserializeStaticBlock(pos) {
  return {
    type: 'StaticBlock',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeVecStatement(pos + 8),
  };
}

function deserializeAccessorProperty(pos) {
  return {
    type: deserializeAccessorPropertyType(pos + 8),
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    key: deserializePropertyKey(pos + 48),
    value: deserializeOptionExpression(pos + 64),
    computed: deserializeBool(pos + 80),
    static: deserializeBool(pos + 81),
    decorators: deserializeVecDecorator(pos + 16),
    definite: deserializeBool(pos + 82),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 88),
    accessibility: deserializeOptionTSAccessibility(pos + 96),
  };
}

function deserializeImportExpression(pos) {
  const options = deserializeVecExpression(pos + 24);
  return {
    type: 'ImportExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    source: deserializeExpression(pos + 8),
    options: options.length === 0 ? null : options[0],
  };
}

function deserializeImportDeclaration(pos) {
  let specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 8);
  if (specifiers === null) specifiers = [];
  const withClause = deserializeOptionBoxWithClause(pos + 96);
  return {
    type: 'ImportDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    specifiers,
    source: deserializeStringLiteral(pos + 40),
    attributes: withClause === null ? [] : withClause.withEntries,
    importKind: deserializeImportOrExportKind(pos + 104),
  };
}

function deserializeImportSpecifier(pos) {
  return {
    type: 'ImportSpecifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    imported: deserializeModuleExportName(pos + 8),
    local: deserializeBindingIdentifier(pos + 64),
    importKind: deserializeImportOrExportKind(pos + 96),
  };
}

function deserializeImportDefaultSpecifier(pos) {
  return {
    type: 'ImportDefaultSpecifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    local: deserializeBindingIdentifier(pos + 8),
  };
}

function deserializeImportNamespaceSpecifier(pos) {
  return {
    type: 'ImportNamespaceSpecifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    local: deserializeBindingIdentifier(pos + 8),
  };
}

function deserializeWithClause(pos) {
  return {
    type: 'WithClause',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    attributesKeyword: deserializeIdentifierName(pos + 8),
    withEntries: deserializeVecImportAttribute(pos + 32),
  };
}

function deserializeImportAttribute(pos) {
  return {
    type: 'ImportAttribute',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    key: deserializeImportAttributeKey(pos + 8),
    value: deserializeStringLiteral(pos + 64),
  };
}

function deserializeExportNamedDeclaration(pos) {
  const withClause = deserializeOptionBoxWithClause(pos + 112);
  return {
    type: 'ExportNamedDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    declaration: deserializeOptionDeclaration(pos + 8),
    specifiers: deserializeVecExportSpecifier(pos + 24),
    source: deserializeOptionStringLiteral(pos + 56),
    attributes: withClause === null ? [] : withClause.withEntries,
    exportKind: deserializeImportOrExportKind(pos + 104),
  };
}

function deserializeExportDefaultDeclaration(pos) {
  return {
    type: 'ExportDefaultDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    declaration: deserializeExportDefaultDeclarationKind(pos + 64),
  };
}

function deserializeExportAllDeclaration(pos) {
  const withClause = deserializeOptionBoxWithClause(pos + 112);
  return {
    type: 'ExportAllDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    exported: deserializeOptionModuleExportName(pos + 8),
    source: deserializeStringLiteral(pos + 64),
    attributes: withClause === null ? [] : withClause.withEntries,
    exportKind: deserializeImportOrExportKind(pos + 120),
  };
}

function deserializeExportSpecifier(pos) {
  return {
    type: 'ExportSpecifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    local: deserializeModuleExportName(pos + 8),
    exported: deserializeModuleExportName(pos + 64),
    exportKind: deserializeImportOrExportKind(pos + 120),
  };
}

function deserializeV8IntrinsicExpression(pos) {
  return {
    type: 'V8IntrinsicExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeIdentifierName(pos + 8),
    arguments: deserializeVecArgument(pos + 32),
  };
}

function deserializeBooleanLiteral(pos) {
  const start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    value = deserializeBool(pos + 8);
  return {
    type: 'Literal',
    start,
    end,
    value,
    raw: (start === 0 && end === 0) ? null : value + '',
  };
}

function deserializeNullLiteral(pos) {
  const start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: 'Literal',
    start,
    end,
    value: null,
    raw: (start === 0 && end === 0) ? null : 'null',
  };
}

function deserializeNumericLiteral(pos) {
  return {
    type: 'Literal',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value: deserializeF64(pos + 8),
    raw: deserializeOptionStr(pos + 16),
  };
}

function deserializeStringLiteral(pos) {
  return {
    type: 'Literal',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value: deserializeStr(pos + 8),
    raw: deserializeOptionStr(pos + 24),
  };
}

function deserializeBigIntLiteral(pos) {
  const raw = deserializeStr(pos + 8),
    bigint = raw.slice(0, -1).replace(/_/g, '');
  return {
    type: 'Literal',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value: BigInt(bigint),
    raw,
    bigint,
  };
}

function deserializeRegExpLiteral(pos) {
  const regex = deserializeRegExp(pos + 8);
  let value = null;
  try {
    value = new RegExp(regex.pattern, regex.flags);
  } catch (e) {}
  return {
    type: 'Literal',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value,
    raw: deserializeOptionStr(pos + 40),
    regex,
  };
}

function deserializeRegExp(pos) {
  return {
    pattern: deserializeRegExpPattern(pos),
    flags: deserializeRegExpFlags(pos + 24),
  };
}

function deserializeRegExpFlags(pos) {
  const flagBits = deserializeU8(pos);
  let flags = '';
  // Alphabetical order
  if (flagBits & 64) flags += 'd';
  if (flagBits & 1) flags += 'g';
  if (flagBits & 2) flags += 'i';
  if (flagBits & 4) flags += 'm';
  if (flagBits & 8) flags += 's';
  if (flagBits & 16) flags += 'u';
  if (flagBits & 128) flags += 'v';
  if (flagBits & 32) flags += 'y';
  return flags;
}

function deserializeJSXElement(pos) {
  return {
    type: 'JSXElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    openingElement: deserializeBoxJSXOpeningElement(pos + 8),
    closingElement: deserializeOptionBoxJSXClosingElement(pos + 16),
    children: deserializeVecJSXChild(pos + 24),
  };
}

function deserializeJSXOpeningElement(pos) {
  return {
    type: 'JSXOpeningElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    attributes: deserializeVecJSXAttributeItem(pos + 32),
    name: deserializeJSXElementName(pos + 16),
    selfClosing: deserializeBool(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 64),
  };
}

function deserializeJSXClosingElement(pos) {
  return {
    type: 'JSXClosingElement',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeJSXElementName(pos + 8),
  };
}

function deserializeJSXFragment(pos) {
  return {
    type: 'JSXFragment',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    openingFragment: deserializeJSXOpeningFragment(pos + 8),
    closingFragment: deserializeJSXClosingFragment(pos + 16),
    children: deserializeVecJSXChild(pos + 24),
  };
}

function deserializeJSXOpeningFragment(pos) {
  return {
    type: 'JSXOpeningFragment',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    attributes: [],
    selfClosing: false,
  };
}

function deserializeJSXClosingFragment(pos) {
  return {
    type: 'JSXClosingFragment',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeJSXNamespacedName(pos) {
  return {
    type: 'JSXNamespacedName',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    namespace: deserializeJSXIdentifier(pos + 8),
    name: deserializeJSXIdentifier(pos + 32),
  };
}

function deserializeJSXMemberExpression(pos) {
  return {
    type: 'JSXMemberExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    object: deserializeJSXMemberExpressionObject(pos + 8),
    property: deserializeJSXIdentifier(pos + 24),
  };
}

function deserializeJSXExpressionContainer(pos) {
  return {
    type: 'JSXExpressionContainer',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeJSXExpression(pos + 8),
  };
}

function deserializeJSXEmptyExpression(pos) {
  return {
    type: 'JSXEmptyExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeJSXAttribute(pos) {
  return {
    type: 'JSXAttribute',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeJSXAttributeName(pos + 8),
    value: deserializeOptionJSXAttributeValue(pos + 24),
  };
}

function deserializeJSXSpreadAttribute(pos) {
  return {
    type: 'JSXSpreadAttribute',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeExpression(pos + 8),
  };
}

function deserializeJSXIdentifier(pos) {
  return {
    type: 'JSXIdentifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
  };
}

function deserializeJSXSpreadChild(pos) {
  return {
    type: 'JSXSpreadChild',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
  };
}

function deserializeJSXText(pos) {
  return {
    type: 'JSXText',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    value: deserializeStr(pos + 8),
    raw: deserializeOptionStr(pos + 24),
  };
}

function deserializeTSThisParameter(pos) {
  return {
    type: 'Identifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: 'this',
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 16),
    decorators: [],
    optional: false,
  };
}

function deserializeTSEnumDeclaration(pos) {
  return {
    type: 'TSEnumDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeBindingIdentifier(pos + 8),
    members: deserializeVecTSEnumMember(pos + 40),
    const: deserializeBool(pos + 72),
    declare: deserializeBool(pos + 73),
  };
}

function deserializeTSEnumMember(pos) {
  return {
    type: 'TSEnumMember',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeTSEnumMemberName(pos + 8),
    initializer: deserializeOptionExpression(pos + 24),
  };
}

function deserializeTSTypeAnnotation(pos) {
  return {
    type: 'TSTypeAnnotation',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeAnnotation: deserializeTSType(pos + 8),
  };
}

function deserializeTSLiteralType(pos) {
  return {
    type: 'TSLiteralType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    literal: deserializeTSLiteral(pos + 8),
  };
}

function deserializeTSConditionalType(pos) {
  return {
    type: 'TSConditionalType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    checkType: deserializeTSType(pos + 8),
    extendsType: deserializeTSType(pos + 24),
    trueType: deserializeTSType(pos + 40),
    falseType: deserializeTSType(pos + 56),
  };
}

function deserializeTSUnionType(pos) {
  return {
    type: 'TSUnionType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    types: deserializeVecTSType(pos + 8),
  };
}

function deserializeTSIntersectionType(pos) {
  return {
    type: 'TSIntersectionType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    types: deserializeVecTSType(pos + 8),
  };
}

function deserializeTSParenthesizedType(pos) {
  return {
    type: 'TSParenthesizedType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeAnnotation: deserializeTSType(pos + 8),
  };
}

function deserializeTSTypeOperator(pos) {
  return {
    type: 'TSTypeOperator',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    operator: deserializeTSTypeOperatorOperator(pos + 8),
    typeAnnotation: deserializeTSType(pos + 16),
  };
}

function deserializeTSArrayType(pos) {
  return {
    type: 'TSArrayType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    elementType: deserializeTSType(pos + 8),
  };
}

function deserializeTSIndexedAccessType(pos) {
  return {
    type: 'TSIndexedAccessType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    objectType: deserializeTSType(pos + 8),
    indexType: deserializeTSType(pos + 24),
  };
}

function deserializeTSTupleType(pos) {
  return {
    type: 'TSTupleType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    elementTypes: deserializeVecTSTupleElement(pos + 8),
  };
}

function deserializeTSNamedTupleMember(pos) {
  return {
    type: 'TSNamedTupleMember',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    elementType: deserializeTSTupleElement(pos + 8),
    label: deserializeIdentifierName(pos + 24),
    optional: deserializeBool(pos + 48),
  };
}

function deserializeTSOptionalType(pos) {
  return {
    type: 'TSOptionalType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeAnnotation: deserializeTSType(pos + 8),
  };
}

function deserializeTSRestType(pos) {
  return {
    type: 'TSRestType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeAnnotation: deserializeTSType(pos + 8),
  };
}

function deserializeTSAnyKeyword(pos) {
  return {
    type: 'TSAnyKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSStringKeyword(pos) {
  return {
    type: 'TSStringKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSBooleanKeyword(pos) {
  return {
    type: 'TSBooleanKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSNumberKeyword(pos) {
  return {
    type: 'TSNumberKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSNeverKeyword(pos) {
  return {
    type: 'TSNeverKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  return {
    type: 'TSIntrinsicKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSUnknownKeyword(pos) {
  return {
    type: 'TSUnknownKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSNullKeyword(pos) {
  return {
    type: 'TSNullKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSUndefinedKeyword(pos) {
  return {
    type: 'TSUndefinedKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSVoidKeyword(pos) {
  return {
    type: 'TSVoidKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSSymbolKeyword(pos) {
  return {
    type: 'TSSymbolKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSThisType(pos) {
  return {
    type: 'TSThisType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSObjectKeyword(pos) {
  return {
    type: 'TSObjectKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSBigIntKeyword(pos) {
  return {
    type: 'TSBigIntKeyword',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSTypeReference(pos) {
  return {
    type: 'TSTypeReference',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeName: deserializeTSTypeName(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
  };
}

function deserializeTSQualifiedName(pos) {
  return {
    type: 'TSQualifiedName',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    left: deserializeTSTypeName(pos + 8),
    right: deserializeIdentifierName(pos + 24),
  };
}

function deserializeTSTypeParameterInstantiation(pos) {
  return {
    type: 'TSTypeParameterInstantiation',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    params: deserializeVecTSType(pos + 8),
  };
}

function deserializeTSTypeParameter(pos) {
  return {
    type: 'TSTypeParameter',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeBindingIdentifier(pos + 8),
    constraint: deserializeOptionTSType(pos + 40),
    default: deserializeOptionTSType(pos + 56),
    in: deserializeBool(pos + 72),
    out: deserializeBool(pos + 73),
    const: deserializeBool(pos + 74),
  };
}

function deserializeTSTypeParameterDeclaration(pos) {
  return {
    type: 'TSTypeParameterDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    params: deserializeVecTSTypeParameter(pos + 8),
  };
}

function deserializeTSTypeAliasDeclaration(pos) {
  return {
    type: 'TSTypeAliasDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeBindingIdentifier(pos + 8),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 40),
    typeAnnotation: deserializeTSType(pos + 48),
    declare: deserializeBool(pos + 64),
  };
}

function deserializeTSClassImplements(pos) {
  return {
    type: 'TSClassImplements',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeTSTypeName(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
  };
}

function deserializeTSInterfaceDeclaration(pos) {
  return {
    type: 'TSInterfaceDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeBindingIdentifier(pos + 8),
    extends: deserializeOptionVecTSInterfaceHeritage(pos + 40),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 72),
    body: deserializeBoxTSInterfaceBody(pos + 80),
    declare: deserializeBool(pos + 88),
  };
}

function deserializeTSInterfaceBody(pos) {
  return {
    type: 'TSInterfaceBody',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeVecTSSignature(pos + 8),
  };
}

function deserializeTSPropertySignature(pos) {
  return {
    type: 'TSPropertySignature',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    computed: deserializeBool(pos + 8),
    optional: deserializeBool(pos + 9),
    readonly: deserializeBool(pos + 10),
    key: deserializePropertyKey(pos + 16),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 32),
  };
}

function deserializeTSIndexSignature(pos) {
  return {
    type: 'TSIndexSignature',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    parameters: deserializeVecTSIndexSignatureName(pos + 8),
    typeAnnotation: deserializeBoxTSTypeAnnotation(pos + 40),
    readonly: deserializeBool(pos + 48),
    static: deserializeBool(pos + 49),
  };
}

function deserializeTSCallSignatureDeclaration(pos) {
  return {
    type: 'TSCallSignatureDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    thisParam: deserializeOptionTSThisParameter(pos + 16),
    params: deserializeBoxFormalParameters(pos + 48),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 56),
  };
}

function deserializeTSMethodSignature(pos) {
  return {
    type: 'TSMethodSignature',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    key: deserializePropertyKey(pos + 8),
    computed: deserializeBool(pos + 24),
    optional: deserializeBool(pos + 25),
    kind: deserializeTSMethodSignatureKind(pos + 26),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 32),
    thisParam: deserializeOptionBoxTSThisParameter(pos + 40),
    params: deserializeBoxFormalParameters(pos + 48),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 56),
  };
}

function deserializeTSConstructSignatureDeclaration(pos) {
  return {
    type: 'TSConstructSignatureDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    params: deserializeBoxFormalParameters(pos + 16),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 24),
  };
}

function deserializeTSIndexSignatureName(pos) {
  return {
    type: 'Identifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
    typeAnnotation: deserializeBoxTSTypeAnnotation(pos + 24),
  };
}

function deserializeTSInterfaceHeritage(pos) {
  return {
    type: 'TSInterfaceHeritage',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
  };
}

function deserializeTSTypePredicate(pos) {
  return {
    type: 'TSTypePredicate',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    parameterName: deserializeTSTypePredicateName(pos + 8),
    asserts: deserializeBool(pos + 24),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 32),
  };
}

function deserializeTSModuleDeclaration(pos) {
  const kind = deserializeTSModuleDeclarationKind(pos + 80);
  return {
    type: 'TSModuleDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeTSModuleDeclarationName(pos + 8),
    body: deserializeOptionTSModuleDeclarationBody(pos + 64),
    kind,
    declare: deserializeBool(pos + 81),
    global: kind === 'global',
  };
}

function deserializeTSModuleBlock(pos) {
  const body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 40));
  return {
    type: 'TSModuleBlock',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body,
  };
}

function deserializeTSTypeLiteral(pos) {
  return {
    type: 'TSTypeLiteral',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    members: deserializeVecTSSignature(pos + 8),
  };
}

function deserializeTSInferType(pos) {
  return {
    type: 'TSInferType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeParameter: deserializeBoxTSTypeParameter(pos + 8),
  };
}

function deserializeTSTypeQuery(pos) {
  return {
    type: 'TSTypeQuery',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    exprName: deserializeTSTypeQueryExprName(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
  };
}

function deserializeTSImportType(pos) {
  return {
    type: 'TSImportType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    argument: deserializeTSType(pos + 8),
    options: deserializeOptionBoxObjectExpression(pos + 24),
    qualifier: deserializeOptionTSTypeName(pos + 32),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 48),
    isTypeOf: deserializeBool(pos + 56),
  };
}

function deserializeTSFunctionType(pos) {
  return {
    type: 'TSFunctionType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    thisParam: deserializeOptionBoxTSThisParameter(pos + 16),
    params: deserializeBoxFormalParameters(pos + 24),
    returnType: deserializeBoxTSTypeAnnotation(pos + 32),
  };
}

function deserializeTSConstructorType(pos) {
  return {
    type: 'TSConstructorType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    abstract: deserializeBool(pos + 8),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 16),
    params: deserializeBoxFormalParameters(pos + 24),
    returnType: deserializeBoxTSTypeAnnotation(pos + 32),
  };
}

function deserializeTSMappedType(pos) {
  return {
    type: 'TSMappedType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeParameter: deserializeBoxTSTypeParameter(pos + 8),
    nameType: deserializeOptionTSType(pos + 16),
    typeAnnotation: deserializeOptionTSType(pos + 32),
    optional: deserializeTSMappedTypeModifierOperator(pos + 48),
    readonly: deserializeTSMappedTypeModifierOperator(pos + 49),
  };
}

function deserializeTSTemplateLiteralType(pos) {
  return {
    type: 'TSTemplateLiteralType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    quasis: deserializeVecTemplateElement(pos + 8),
    types: deserializeVecTSType(pos + 40),
  };
}

function deserializeTSAsExpression(pos) {
  return {
    type: 'TSAsExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
    typeAnnotation: deserializeTSType(pos + 24),
  };
}

function deserializeTSSatisfiesExpression(pos) {
  return {
    type: 'TSSatisfiesExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
    typeAnnotation: deserializeTSType(pos + 24),
  };
}

function deserializeTSTypeAssertion(pos) {
  return {
    type: 'TSTypeAssertion',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
    typeAnnotation: deserializeTSType(pos + 24),
  };
}

function deserializeTSImportEqualsDeclaration(pos) {
  return {
    type: 'TSImportEqualsDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeBindingIdentifier(pos + 8),
    moduleReference: deserializeTSModuleReference(pos + 40),
    importKind: deserializeImportOrExportKind(pos + 56),
  };
}

function deserializeTSExternalModuleReference(pos) {
  return {
    type: 'TSExternalModuleReference',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeStringLiteral(pos + 8),
  };
}

function deserializeTSNonNullExpression(pos) {
  return {
    type: 'TSNonNullExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
  };
}

function deserializeDecorator(pos) {
  return {
    type: 'Decorator',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
  };
}

function deserializeTSExportAssignment(pos) {
  return {
    type: 'TSExportAssignment',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
  };
}

function deserializeTSNamespaceExportDeclaration(pos) {
  return {
    type: 'TSNamespaceExportDeclaration',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    id: deserializeIdentifierName(pos + 8),
  };
}

function deserializeTSInstantiationExpression(pos) {
  return {
    type: 'TSInstantiationExpression',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    expression: deserializeExpression(pos + 8),
    typeParameters: deserializeBoxTSTypeParameterInstantiation(pos + 24),
  };
}

function deserializeJSDocNullableType(pos) {
  return {
    type: 'JSDocNullableType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeAnnotation: deserializeTSType(pos + 8),
    postfix: deserializeBool(pos + 24),
  };
}

function deserializeJSDocNonNullableType(pos) {
  return {
    type: 'JSDocNonNullableType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    typeAnnotation: deserializeTSType(pos + 8),
    postfix: deserializeBool(pos + 24),
  };
}

function deserializeJSDocUnknownType(pos) {
  return {
    type: 'JSDocUnknownType',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeComment(pos) {
  const type = deserializeCommentKind(pos + 12),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  const endCut = type === 'Line' ? 0 : 2;
  return {
    type,
    value: sourceText.slice(start + 2, end - endCut),
    start,
    end,
  };
}

function deserializeNameSpan(pos) {
  return {
    value: deserializeStr(pos),
    start: deserializeU32(pos + 16),
    end: deserializeU32(pos + 20),
  };
}

function deserializeImportEntry(pos) {
  return {
    importName: deserializeImportImportName(pos + 32),
    localName: deserializeNameSpan(pos + 64),
    isType: deserializeBool(pos + 88),
  };
}

function deserializeExportEntry(pos) {
  return {
    start: deserializeU32(pos + 8),
    end: deserializeU32(pos + 12),
    moduleRequest: deserializeOptionNameSpan(pos + 16),
    importName: deserializeExportImportName(pos + 40),
    exportName: deserializeExportExportName(pos + 72),
    localName: deserializeExportLocalName(pos + 104),
  };
}

function deserializeDynamicImport(pos) {
  return {
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    moduleRequest: deserializeSpan(pos + 8),
  };
}

function deserializeSpan(pos) {
  return {
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeSourceType(pos) {
  return {
    sourceType: deserializeModuleKind(pos + 1),
  };
}

function deserializePattern(pos) {
  return {
    type: 'Pattern',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeDisjunction(pos + 8),
  };
}

function deserializeDisjunction(pos) {
  return {
    type: 'Disjunction',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeVecAlternative(pos + 8),
  };
}

function deserializeAlternative(pos) {
  return {
    type: 'Alternative',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    body: deserializeVecTerm(pos + 8),
  };
}

function deserializeBoundaryAssertion(pos) {
  return {
    type: 'BoundaryAssertion',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    kind: deserializeBoundaryAssertionKind(pos + 8),
  };
}

function deserializeLookAroundAssertion(pos) {
  return {
    type: 'LookAroundAssertion',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    kind: deserializeLookAroundAssertionKind(pos + 8),
    body: deserializeDisjunction(pos + 16),
  };
}

function deserializeQuantifier(pos) {
  return {
    type: 'Quantifier',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    min: deserializeU64(pos + 8),
    max: deserializeOptionU64(pos + 16),
    greedy: deserializeBool(pos + 32),
    body: deserializeTerm(pos + 40),
  };
}

function deserializeCharacter(pos) {
  return {
    type: 'Character',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    kind: deserializeCharacterKind(pos + 8),
    value: deserializeU32(pos + 12),
  };
}

function deserializeCharacterClassEscape(pos) {
  return {
    type: 'CharacterClassEscape',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    kind: deserializeCharacterClassEscapeKind(pos + 8),
  };
}

function deserializeUnicodePropertyEscape(pos) {
  return {
    type: 'UnicodePropertyEscape',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    negative: deserializeBool(pos + 8),
    strings: deserializeBool(pos + 9),
    name: deserializeStr(pos + 16),
    value: deserializeOptionStr(pos + 32),
  };
}

function deserializeDot(pos) {
  return {
    type: 'Dot',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeCharacterClass(pos) {
  return {
    type: 'CharacterClass',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    negative: deserializeBool(pos + 8),
    strings: deserializeBool(pos + 9),
    kind: deserializeCharacterClassContentsKind(pos + 10),
    body: deserializeVecCharacterClassContents(pos + 16),
  };
}

function deserializeCharacterClassRange(pos) {
  return {
    type: 'CharacterClassRange',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    min: deserializeCharacter(pos + 8),
    max: deserializeCharacter(pos + 24),
  };
}

function deserializeClassStringDisjunction(pos) {
  return {
    type: 'ClassStringDisjunction',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    strings: deserializeBool(pos + 8),
    body: deserializeVecClassString(pos + 16),
  };
}

function deserializeClassString(pos) {
  return {
    type: 'ClassString',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    strings: deserializeBool(pos + 8),
    body: deserializeVecCharacter(pos + 16),
  };
}

function deserializeCapturingGroup(pos) {
  return {
    type: 'CapturingGroup',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeOptionStr(pos + 8),
    body: deserializeDisjunction(pos + 24),
  };
}

function deserializeIgnoreGroup(pos) {
  return {
    type: 'IgnoreGroup',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    modifiers: deserializeOptionModifiers(pos + 8),
    body: deserializeDisjunction(pos + 24),
  };
}

function deserializeModifiers(pos) {
  return {
    type: 'Modifiers',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    enabling: deserializeOptionModifier(pos + 8),
    disabling: deserializeOptionModifier(pos + 11),
  };
}

function deserializeModifier(pos) {
  return {
    type: 'Modifier',
    ignoreCase: deserializeBool(pos),
    multiline: deserializeBool(pos + 1),
    sticky: deserializeBool(pos + 2),
  };
}

function deserializeIndexedReference(pos) {
  return {
    type: 'IndexedReference',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    index: deserializeU32(pos + 8),
  };
}

function deserializeNamedReference(pos) {
  return {
    type: 'NamedReference',
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    name: deserializeStr(pos + 8),
  };
}

function deserializeRawTransferData(pos) {
  return {
    program: deserializeProgram(pos),
    comments: deserializeVecComment(pos + 160),
    module: deserializeEcmaScriptModule(pos + 192),
    errors: deserializeVecError(pos + 328),
  };
}

function deserializeError(pos) {
  return {
    severity: deserializeErrorSeverity(pos),
    message: deserializeStr(pos + 8),
    labels: deserializeVecErrorLabel(pos + 24),
    helpMessage: deserializeOptionStr(pos + 56),
  };
}

function deserializeErrorLabel(pos) {
  return {
    message: deserializeOptionStr(pos),
    start: deserializeU32(pos + 16),
    end: deserializeU32(pos + 20),
  };
}

function deserializeEcmaScriptModule(pos) {
  return {
    hasModuleSyntax: deserializeBool(pos),
    staticImports: deserializeVecStaticImport(pos + 8),
    staticExports: deserializeVecStaticExport(pos + 40),
    dynamicImports: deserializeVecDynamicImport(pos + 72),
    importMetas: deserializeVecSpan(pos + 104),
  };
}

function deserializeStaticImport(pos) {
  return {
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    moduleRequest: deserializeNameSpan(pos + 8),
    entries: deserializeVecImportEntry(pos + 32),
  };
}

function deserializeStaticExport(pos) {
  return {
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
    entries: deserializeVecExportEntry(pos + 8),
  };
}

function deserializeExpression(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for Expression`);
  }
}

function deserializeArrayExpressionElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    case 64:
      return deserializeBoxSpreadElement(pos + 8);
    case 65:
      return deserializeElision(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ArrayExpressionElement`);
  }
}

function deserializeObjectPropertyKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxObjectProperty(pos + 8);
    case 1:
      return deserializeBoxSpreadElement(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ObjectPropertyKind`);
  }
}

function deserializePropertyKey(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    case 64:
      return deserializeBoxIdentifierName(pos + 8);
    case 65:
      return deserializeBoxPrivateIdentifier(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for PropertyKey`);
  }
}

function deserializePropertyKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'init';
    case 1:
      return 'get';
    case 2:
      return 'set';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for PropertyKind`);
  }
}

function deserializeMemberExpression(pos) {
  switch (uint8[pos]) {
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for MemberExpression`);
  }
}

function deserializeArgument(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    case 64:
      return deserializeBoxSpreadElement(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for Argument`);
  }
}

function deserializeAssignmentTarget(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSAsExpression(pos + 8);
    case 2:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 3:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 4:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 8:
      return deserializeBoxArrayAssignmentTarget(pos + 8);
    case 9:
      return deserializeBoxObjectAssignmentTarget(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTarget`);
  }
}

function deserializeSimpleAssignmentTarget(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSAsExpression(pos + 8);
    case 2:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 3:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 4:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for SimpleAssignmentTarget`);
  }
}

function deserializeAssignmentTargetPattern(pos) {
  switch (uint8[pos]) {
    case 8:
      return deserializeBoxArrayAssignmentTarget(pos + 8);
    case 9:
      return deserializeBoxObjectAssignmentTarget(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetPattern`);
  }
}

function deserializeAssignmentTargetMaybeDefault(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSAsExpression(pos + 8);
    case 2:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 3:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 4:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 8:
      return deserializeBoxArrayAssignmentTarget(pos + 8);
    case 9:
      return deserializeBoxObjectAssignmentTarget(pos + 8);
    case 16:
      return deserializeBoxAssignmentTargetWithDefault(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetMaybeDefault`);
  }
}

function deserializeAssignmentTargetProperty(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxAssignmentTargetPropertyIdentifier(pos + 8);
    case 1:
      return deserializeBoxAssignmentTargetPropertyProperty(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetProperty`);
  }
}

function deserializeChainElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxCallExpression(pos + 8);
    case 1:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ChainElement`);
  }
}

function deserializeStatement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBlockStatement(pos + 8);
    case 1:
      return deserializeBoxBreakStatement(pos + 8);
    case 2:
      return deserializeBoxContinueStatement(pos + 8);
    case 3:
      return deserializeBoxDebuggerStatement(pos + 8);
    case 4:
      return deserializeBoxDoWhileStatement(pos + 8);
    case 5:
      return deserializeBoxEmptyStatement(pos + 8);
    case 6:
      return deserializeBoxExpressionStatement(pos + 8);
    case 7:
      return deserializeBoxForInStatement(pos + 8);
    case 8:
      return deserializeBoxForOfStatement(pos + 8);
    case 9:
      return deserializeBoxForStatement(pos + 8);
    case 10:
      return deserializeBoxIfStatement(pos + 8);
    case 11:
      return deserializeBoxLabeledStatement(pos + 8);
    case 12:
      return deserializeBoxReturnStatement(pos + 8);
    case 13:
      return deserializeBoxSwitchStatement(pos + 8);
    case 14:
      return deserializeBoxThrowStatement(pos + 8);
    case 15:
      return deserializeBoxTryStatement(pos + 8);
    case 16:
      return deserializeBoxWhileStatement(pos + 8);
    case 17:
      return deserializeBoxWithStatement(pos + 8);
    case 32:
      return deserializeBoxVariableDeclaration(pos + 8);
    case 33:
      return deserializeBoxFunction(pos + 8);
    case 34:
      return deserializeBoxClass(pos + 8);
    case 35:
      return deserializeBoxTSTypeAliasDeclaration(pos + 8);
    case 36:
      return deserializeBoxTSInterfaceDeclaration(pos + 8);
    case 37:
      return deserializeBoxTSEnumDeclaration(pos + 8);
    case 38:
      return deserializeBoxTSModuleDeclaration(pos + 8);
    case 39:
      return deserializeBoxTSImportEqualsDeclaration(pos + 8);
    case 64:
      return deserializeBoxImportDeclaration(pos + 8);
    case 65:
      return deserializeBoxExportAllDeclaration(pos + 8);
    case 66:
      return deserializeBoxExportDefaultDeclaration(pos + 8);
    case 67:
      return deserializeBoxExportNamedDeclaration(pos + 8);
    case 68:
      return deserializeBoxTSExportAssignment(pos + 8);
    case 69:
      return deserializeBoxTSNamespaceExportDeclaration(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for Statement`);
  }
}

function deserializeDeclaration(pos) {
  switch (uint8[pos]) {
    case 32:
      return deserializeBoxVariableDeclaration(pos + 8);
    case 33:
      return deserializeBoxFunction(pos + 8);
    case 34:
      return deserializeBoxClass(pos + 8);
    case 35:
      return deserializeBoxTSTypeAliasDeclaration(pos + 8);
    case 36:
      return deserializeBoxTSInterfaceDeclaration(pos + 8);
    case 37:
      return deserializeBoxTSEnumDeclaration(pos + 8);
    case 38:
      return deserializeBoxTSModuleDeclaration(pos + 8);
    case 39:
      return deserializeBoxTSImportEqualsDeclaration(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for Declaration`);
  }
}

function deserializeVariableDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'var';
    case 1:
      return 'let';
    case 2:
      return 'const';
    case 3:
      return 'using';
    case 4:
      return 'await using';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for VariableDeclarationKind`);
  }
}

function deserializeForStatementInit(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    case 64:
      return deserializeBoxVariableDeclaration(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ForStatementInit`);
  }
}

function deserializeForStatementLeft(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSAsExpression(pos + 8);
    case 2:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 3:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 4:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 8:
      return deserializeBoxArrayAssignmentTarget(pos + 8);
    case 9:
      return deserializeBoxObjectAssignmentTarget(pos + 8);
    case 16:
      return deserializeBoxVariableDeclaration(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ForStatementLeft`);
  }
}

function deserializeBindingPatternKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBindingIdentifier(pos + 8);
    case 1:
      return deserializeBoxObjectPattern(pos + 8);
    case 2:
      return deserializeBoxArrayPattern(pos + 8);
    case 3:
      return deserializeBoxAssignmentPattern(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for BindingPatternKind`);
  }
}

function deserializeFunctionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'FunctionDeclaration';
    case 1:
      return 'FunctionExpression';
    case 2:
      return 'TSDeclareFunction';
    case 3:
      return 'TSEmptyBodyFunctionExpression';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for FunctionType`);
  }
}

function deserializeFormalParameterKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'FormalParameter';
    case 1:
      return 'UniqueFormalParameters';
    case 2:
      return 'ArrowFormalParameters';
    case 3:
      return 'Signature';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for FormalParameterKind`);
  }
}

function deserializeClassType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'ClassDeclaration';
    case 1:
      return 'ClassExpression';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ClassType`);
  }
}

function deserializeClassElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxStaticBlock(pos + 8);
    case 1:
      return deserializeBoxMethodDefinition(pos + 8);
    case 2:
      return deserializeBoxPropertyDefinition(pos + 8);
    case 3:
      return deserializeBoxAccessorProperty(pos + 8);
    case 4:
      return deserializeBoxTSIndexSignature(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ClassElement`);
  }
}

function deserializeMethodDefinitionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'MethodDefinition';
    case 1:
      return 'TSAbstractMethodDefinition';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for MethodDefinitionType`);
  }
}

function deserializePropertyDefinitionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'PropertyDefinition';
    case 1:
      return 'TSAbstractPropertyDefinition';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for PropertyDefinitionType`);
  }
}

function deserializeMethodDefinitionKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'constructor';
    case 1:
      return 'method';
    case 2:
      return 'get';
    case 3:
      return 'set';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for MethodDefinitionKind`);
  }
}

function deserializeModuleDeclaration(pos) {
  switch (uint8[pos]) {
    case 64:
      return deserializeBoxImportDeclaration(pos + 8);
    case 65:
      return deserializeBoxExportAllDeclaration(pos + 8);
    case 66:
      return deserializeBoxExportDefaultDeclaration(pos + 8);
    case 67:
      return deserializeBoxExportNamedDeclaration(pos + 8);
    case 68:
      return deserializeBoxTSExportAssignment(pos + 8);
    case 69:
      return deserializeBoxTSNamespaceExportDeclaration(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ModuleDeclaration`);
  }
}

function deserializeAccessorPropertyType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'AccessorProperty';
    case 1:
      return 'TSAbstractAccessorProperty';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for AccessorPropertyType`);
  }
}

function deserializeImportPhase(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'source';
    case 1:
      return 'defer';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ImportPhase`);
  }
}

function deserializeImportDeclarationSpecifier(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxImportSpecifier(pos + 8);
    case 1:
      return deserializeBoxImportDefaultSpecifier(pos + 8);
    case 2:
      return deserializeBoxImportNamespaceSpecifier(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ImportDeclarationSpecifier`);
  }
}

function deserializeImportAttributeKey(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(pos + 8);
    case 1:
      return deserializeStringLiteral(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ImportAttributeKey`);
  }
}

function deserializeExportDefaultDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    case 64:
      return deserializeBoxFunction(pos + 8);
    case 65:
      return deserializeBoxClass(pos + 8);
    case 66:
      return deserializeBoxTSInterfaceDeclaration(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ExportDefaultDeclarationKind`);
  }
}

function deserializeModuleExportName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(pos + 8);
    case 1:
      return deserializeIdentifierReference(pos + 8);
    case 2:
      return deserializeStringLiteral(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ModuleExportName`);
  }
}

function deserializeRegExpPattern(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeStr(pos + 8);
    case 1:
      return deserializeStr(pos + 8);
    case 2:
      return deserializeBoxPattern(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for RegExpPattern`);
  }
}

function deserializeJSXElementName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXIdentifier(pos + 8);
    case 1:
      const ident = deserializeBoxIdentifierReference(pos + 8);
      return { type: 'JSXIdentifier', start: ident.start, end: ident.end, name: ident.name };
    case 2:
      return deserializeBoxJSXNamespacedName(pos + 8);
    case 3:
      return deserializeBoxJSXMemberExpression(pos + 8);
    case 4:
      const thisExpr = deserializeBoxThisExpression(pos + 8);
      return { type: 'JSXIdentifier', start: thisExpr.start, end: thisExpr.end, name: 'this' };
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXMemberExpressionObject(pos) {
  switch (uint8[pos]) {
    case 0:
      const ident = deserializeBoxIdentifierReference(pos + 8);
      return { type: 'JSXIdentifier', start: ident.start, end: ident.end, name: ident.name };
    case 1:
      return deserializeBoxJSXMemberExpression(pos + 8);
    case 2:
      const thisExpr = deserializeBoxThisExpression(pos + 8);
      return { type: 'JSXIdentifier', start: thisExpr.start, end: thisExpr.end, name: 'this' };
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpression(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNullLiteral(pos + 8);
    case 2:
      return deserializeBoxNumericLiteral(pos + 8);
    case 3:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 4:
      return deserializeBoxRegExpLiteral(pos + 8);
    case 5:
      return deserializeBoxStringLiteral(pos + 8);
    case 6:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 7:
      return deserializeBoxIdentifierReference(pos + 8);
    case 8:
      return deserializeBoxMetaProperty(pos + 8);
    case 9:
      return deserializeBoxSuper(pos + 8);
    case 10:
      return deserializeBoxArrayExpression(pos + 8);
    case 11:
      return deserializeBoxArrowFunctionExpression(pos + 8);
    case 12:
      return deserializeBoxAssignmentExpression(pos + 8);
    case 13:
      return deserializeBoxAwaitExpression(pos + 8);
    case 14:
      return deserializeBoxBinaryExpression(pos + 8);
    case 15:
      return deserializeBoxCallExpression(pos + 8);
    case 16:
      return deserializeBoxChainExpression(pos + 8);
    case 17:
      return deserializeBoxClass(pos + 8);
    case 18:
      return deserializeBoxConditionalExpression(pos + 8);
    case 19:
      return deserializeBoxFunction(pos + 8);
    case 20:
      return deserializeBoxImportExpression(pos + 8);
    case 21:
      return deserializeBoxLogicalExpression(pos + 8);
    case 22:
      return deserializeBoxNewExpression(pos + 8);
    case 23:
      return deserializeBoxObjectExpression(pos + 8);
    case 24:
      return deserializeBoxParenthesizedExpression(pos + 8);
    case 25:
      return deserializeBoxSequenceExpression(pos + 8);
    case 26:
      return deserializeBoxTaggedTemplateExpression(pos + 8);
    case 27:
      return deserializeBoxThisExpression(pos + 8);
    case 28:
      return deserializeBoxUnaryExpression(pos + 8);
    case 29:
      return deserializeBoxUpdateExpression(pos + 8);
    case 30:
      return deserializeBoxYieldExpression(pos + 8);
    case 31:
      return deserializeBoxPrivateInExpression(pos + 8);
    case 32:
      return deserializeBoxJSXElement(pos + 8);
    case 33:
      return deserializeBoxJSXFragment(pos + 8);
    case 34:
      return deserializeBoxTSAsExpression(pos + 8);
    case 35:
      return deserializeBoxTSSatisfiesExpression(pos + 8);
    case 36:
      return deserializeBoxTSTypeAssertion(pos + 8);
    case 37:
      return deserializeBoxTSNonNullExpression(pos + 8);
    case 38:
      return deserializeBoxTSInstantiationExpression(pos + 8);
    case 39:
      return deserializeBoxV8IntrinsicExpression(pos + 8);
    case 48:
      return deserializeBoxComputedMemberExpression(pos + 8);
    case 49:
      return deserializeBoxStaticMemberExpression(pos + 8);
    case 50:
      return deserializeBoxPrivateFieldExpression(pos + 8);
    case 64:
      return deserializeJSXEmptyExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXExpression`);
  }
}

function deserializeJSXAttributeItem(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXAttribute(pos + 8);
    case 1:
      return deserializeBoxJSXSpreadAttribute(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeItem`);
  }
}

function deserializeJSXAttributeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXIdentifier(pos + 8);
    case 1:
      return deserializeBoxJSXNamespacedName(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeName`);
  }
}

function deserializeJSXAttributeValue(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxStringLiteral(pos + 8);
    case 1:
      return deserializeBoxJSXExpressionContainer(pos + 8);
    case 2:
      return deserializeBoxJSXElement(pos + 8);
    case 3:
      return deserializeBoxJSXFragment(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeValue`);
  }
}

function deserializeJSXChild(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXText(pos + 8);
    case 1:
      return deserializeBoxJSXElement(pos + 8);
    case 2:
      return deserializeBoxJSXFragment(pos + 8);
    case 3:
      return deserializeBoxJSXExpressionContainer(pos + 8);
    case 4:
      return deserializeBoxJSXSpreadChild(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for JSXChild`);
  }
}

function deserializeTSEnumMemberName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierName(pos + 8);
    case 1:
      return deserializeBoxStringLiteral(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSEnumMemberName`);
  }
}

function deserializeTSLiteral(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBooleanLiteral(pos + 8);
    case 1:
      return deserializeBoxNumericLiteral(pos + 8);
    case 2:
      return deserializeBoxBigIntLiteral(pos + 8);
    case 3:
      return deserializeBoxStringLiteral(pos + 8);
    case 4:
      return deserializeBoxTemplateLiteral(pos + 8);
    case 5:
      return deserializeBoxUnaryExpression(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSLiteral`);
  }
}

function deserializeTSType(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxTSAnyKeyword(pos + 8);
    case 1:
      return deserializeBoxTSBigIntKeyword(pos + 8);
    case 2:
      return deserializeBoxTSBooleanKeyword(pos + 8);
    case 3:
      return deserializeBoxTSIntrinsicKeyword(pos + 8);
    case 4:
      return deserializeBoxTSNeverKeyword(pos + 8);
    case 5:
      return deserializeBoxTSNullKeyword(pos + 8);
    case 6:
      return deserializeBoxTSNumberKeyword(pos + 8);
    case 7:
      return deserializeBoxTSObjectKeyword(pos + 8);
    case 8:
      return deserializeBoxTSStringKeyword(pos + 8);
    case 9:
      return deserializeBoxTSSymbolKeyword(pos + 8);
    case 10:
      return deserializeBoxTSThisType(pos + 8);
    case 11:
      return deserializeBoxTSUndefinedKeyword(pos + 8);
    case 12:
      return deserializeBoxTSUnknownKeyword(pos + 8);
    case 13:
      return deserializeBoxTSVoidKeyword(pos + 8);
    case 14:
      return deserializeBoxTSArrayType(pos + 8);
    case 15:
      return deserializeBoxTSConditionalType(pos + 8);
    case 16:
      return deserializeBoxTSConstructorType(pos + 8);
    case 17:
      return deserializeBoxTSFunctionType(pos + 8);
    case 18:
      return deserializeBoxTSImportType(pos + 8);
    case 19:
      return deserializeBoxTSIndexedAccessType(pos + 8);
    case 20:
      return deserializeBoxTSInferType(pos + 8);
    case 21:
      return deserializeBoxTSIntersectionType(pos + 8);
    case 22:
      return deserializeBoxTSLiteralType(pos + 8);
    case 23:
      return deserializeBoxTSMappedType(pos + 8);
    case 24:
      return deserializeBoxTSNamedTupleMember(pos + 8);
    case 26:
      return deserializeBoxTSTemplateLiteralType(pos + 8);
    case 27:
      return deserializeBoxTSTupleType(pos + 8);
    case 28:
      return deserializeBoxTSTypeLiteral(pos + 8);
    case 29:
      return deserializeBoxTSTypeOperator(pos + 8);
    case 30:
      return deserializeBoxTSTypePredicate(pos + 8);
    case 31:
      return deserializeBoxTSTypeQuery(pos + 8);
    case 32:
      return deserializeBoxTSTypeReference(pos + 8);
    case 33:
      return deserializeBoxTSUnionType(pos + 8);
    case 34:
      return deserializeBoxTSParenthesizedType(pos + 8);
    case 35:
      return deserializeBoxJSDocNullableType(pos + 8);
    case 36:
      return deserializeBoxJSDocNonNullableType(pos + 8);
    case 37:
      return deserializeBoxJSDocUnknownType(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSType`);
  }
}

function deserializeTSTypeOperatorOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'keyof';
    case 1:
      return 'unique';
    case 2:
      return 'readonly';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSTypeOperatorOperator`);
  }
}

function deserializeTSTupleElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxTSAnyKeyword(pos + 8);
    case 1:
      return deserializeBoxTSBigIntKeyword(pos + 8);
    case 2:
      return deserializeBoxTSBooleanKeyword(pos + 8);
    case 3:
      return deserializeBoxTSIntrinsicKeyword(pos + 8);
    case 4:
      return deserializeBoxTSNeverKeyword(pos + 8);
    case 5:
      return deserializeBoxTSNullKeyword(pos + 8);
    case 6:
      return deserializeBoxTSNumberKeyword(pos + 8);
    case 7:
      return deserializeBoxTSObjectKeyword(pos + 8);
    case 8:
      return deserializeBoxTSStringKeyword(pos + 8);
    case 9:
      return deserializeBoxTSSymbolKeyword(pos + 8);
    case 10:
      return deserializeBoxTSThisType(pos + 8);
    case 11:
      return deserializeBoxTSUndefinedKeyword(pos + 8);
    case 12:
      return deserializeBoxTSUnknownKeyword(pos + 8);
    case 13:
      return deserializeBoxTSVoidKeyword(pos + 8);
    case 14:
      return deserializeBoxTSArrayType(pos + 8);
    case 15:
      return deserializeBoxTSConditionalType(pos + 8);
    case 16:
      return deserializeBoxTSConstructorType(pos + 8);
    case 17:
      return deserializeBoxTSFunctionType(pos + 8);
    case 18:
      return deserializeBoxTSImportType(pos + 8);
    case 19:
      return deserializeBoxTSIndexedAccessType(pos + 8);
    case 20:
      return deserializeBoxTSInferType(pos + 8);
    case 21:
      return deserializeBoxTSIntersectionType(pos + 8);
    case 22:
      return deserializeBoxTSLiteralType(pos + 8);
    case 23:
      return deserializeBoxTSMappedType(pos + 8);
    case 24:
      return deserializeBoxTSNamedTupleMember(pos + 8);
    case 26:
      return deserializeBoxTSTemplateLiteralType(pos + 8);
    case 27:
      return deserializeBoxTSTupleType(pos + 8);
    case 28:
      return deserializeBoxTSTypeLiteral(pos + 8);
    case 29:
      return deserializeBoxTSTypeOperator(pos + 8);
    case 30:
      return deserializeBoxTSTypePredicate(pos + 8);
    case 31:
      return deserializeBoxTSTypeQuery(pos + 8);
    case 32:
      return deserializeBoxTSTypeReference(pos + 8);
    case 33:
      return deserializeBoxTSUnionType(pos + 8);
    case 34:
      return deserializeBoxTSParenthesizedType(pos + 8);
    case 35:
      return deserializeBoxJSDocNullableType(pos + 8);
    case 36:
      return deserializeBoxJSDocNonNullableType(pos + 8);
    case 37:
      return deserializeBoxJSDocUnknownType(pos + 8);
    case 64:
      return deserializeBoxTSOptionalType(pos + 8);
    case 65:
      return deserializeBoxTSRestType(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSTupleElement`);
  }
}

function deserializeTSTypeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSQualifiedName(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSTypeName`);
  }
}

function deserializeTSAccessibility(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'private';
    case 1:
      return 'protected';
    case 2:
      return 'public';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSAccessibility`);
  }
}

function deserializeTSSignature(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxTSIndexSignature(pos + 8);
    case 1:
      return deserializeBoxTSPropertySignature(pos + 8);
    case 2:
      return deserializeBoxTSCallSignatureDeclaration(pos + 8);
    case 3:
      return deserializeBoxTSConstructSignatureDeclaration(pos + 8);
    case 4:
      return deserializeBoxTSMethodSignature(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSSignature`);
  }
}

function deserializeTSMethodSignatureKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'method';
    case 1:
      return 'get';
    case 2:
      return 'set';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSMethodSignatureKind`);
  }
}

function deserializeTSTypePredicateName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierName(pos + 8);
    case 1:
      return deserializeTSThisType(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSTypePredicateName`);
  }
}

function deserializeTSModuleDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'global';
    case 1:
      return 'module';
    case 2:
      return 'namespace';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSModuleDeclarationKind`);
  }
}

function deserializeTSModuleDeclarationName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBindingIdentifier(pos + 8);
    case 1:
      return deserializeStringLiteral(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSModuleDeclarationName`);
  }
}

function deserializeTSModuleDeclarationBody(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxTSModuleDeclaration(pos + 8);
    case 1:
      return deserializeBoxTSModuleBlock(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSModuleDeclarationBody`);
  }
}

function deserializeTSTypeQueryExprName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSQualifiedName(pos + 8);
    case 2:
      return deserializeBoxTSImportType(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSTypeQueryExprName`);
  }
}

function deserializeTSMappedTypeModifierOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'true';
    case 1:
      return '+';
    case 2:
      return '-';
    case 3:
      return 'none';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSMappedTypeModifierOperator`);
  }
}

function deserializeTSModuleReference(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSQualifiedName(pos + 8);
    case 2:
      return deserializeBoxTSExternalModuleReference(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for TSModuleReference`);
  }
}

function deserializeImportOrExportKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'value';
    case 1:
      return 'type';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ImportOrExportKind`);
  }
}

function deserializeCommentKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'Line';
    case 1:
      return 'Block';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for CommentKind`);
  }
}

function deserializeImportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return { kind: 'Name', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end };
    case 1:
      return { kind: 'NamespaceObject', name: null, start: null, end: null };
    case 2:
      var span = deserializeSpan(pos + 8);
      return { kind: 'Default', name: null, start: span.start, end: span.end };
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ImportImportName`);
  }
}

function deserializeExportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return { kind: 'Name', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end };
    case 1:
      return { kind: 'All', name: null, start: null, end: null };
    case 2:
      return { kind: 'AllButDefault', name: null, start: null, end: null };
    case 3:
      return { kind: 'None', name: null, start: null, end: null };
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ExportImportName`);
  }
}

function deserializeExportExportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return { kind: 'Name', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end };
    case 1:
      var span = deserializeSpan(pos + 8);
      return { kind: 'Default', name: null, start: span.start, end: span.end };
    case 2:
      return { kind: 'None', name: null, start: null, end: null };
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ExportExportName`);
  }
}

function deserializeExportLocalName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return { kind: 'Name', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end };
    case 1:
      var nameSpan = deserializeNameSpan(pos + 8);
      return { kind: 'Default', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end };
    case 2:
      return { kind: 'None', name: null, start: null, end: null };
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ExportLocalName`);
  }
}

function deserializeAssignmentOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return '=';
    case 1:
      return '+=';
    case 2:
      return '-=';
    case 3:
      return '*=';
    case 4:
      return '/=';
    case 5:
      return '%=';
    case 6:
      return '**=';
    case 7:
      return '<<=';
    case 8:
      return '>>=';
    case 9:
      return '>>>=';
    case 10:
      return '|=';
    case 11:
      return '^=';
    case 12:
      return '&=';
    case 13:
      return '||=';
    case 14:
      return '&&=';
    case 15:
      return '??=';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for AssignmentOperator`);
  }
}

function deserializeBinaryOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return '==';
    case 1:
      return '!=';
    case 2:
      return '===';
    case 3:
      return '!==';
    case 4:
      return '<';
    case 5:
      return '<=';
    case 6:
      return '>';
    case 7:
      return '>=';
    case 8:
      return '+';
    case 9:
      return '-';
    case 10:
      return '*';
    case 11:
      return '/';
    case 12:
      return '%';
    case 13:
      return '**';
    case 14:
      return '<<';
    case 15:
      return '>>';
    case 16:
      return '>>>';
    case 17:
      return '|';
    case 18:
      return '^';
    case 19:
      return '&';
    case 20:
      return 'in';
    case 21:
      return 'instanceof';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for BinaryOperator`);
  }
}

function deserializeLogicalOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return '||';
    case 1:
      return '&&';
    case 2:
      return '??';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for LogicalOperator`);
  }
}

function deserializeUnaryOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return '+';
    case 1:
      return '-';
    case 2:
      return '!';
    case 3:
      return '~';
    case 4:
      return 'typeof';
    case 5:
      return 'void';
    case 6:
      return 'delete';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for UnaryOperator`);
  }
}

function deserializeUpdateOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return '++';
    case 1:
      return '--';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for UpdateOperator`);
  }
}

function deserializeModuleKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'script';
    case 1:
      return 'module';
    case 2:
      return 'unambiguous';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ModuleKind`);
  }
}

function deserializeTerm(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxBoundaryAssertion(pos + 8);
    case 1:
      return deserializeBoxLookAroundAssertion(pos + 8);
    case 2:
      return deserializeBoxQuantifier(pos + 8);
    case 3:
      return deserializeBoxCharacter(pos + 8);
    case 4:
      return deserializeDot(pos + 8);
    case 5:
      return deserializeBoxCharacterClassEscape(pos + 8);
    case 6:
      return deserializeBoxUnicodePropertyEscape(pos + 8);
    case 7:
      return deserializeBoxCharacterClass(pos + 8);
    case 8:
      return deserializeBoxCapturingGroup(pos + 8);
    case 9:
      return deserializeBoxIgnoreGroup(pos + 8);
    case 10:
      return deserializeBoxIndexedReference(pos + 8);
    case 11:
      return deserializeBoxNamedReference(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for Term`);
  }
}

function deserializeBoundaryAssertionKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'start';
    case 1:
      return 'end';
    case 2:
      return 'boundary';
    case 3:
      return 'negativeBoundary';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for BoundaryAssertionKind`);
  }
}

function deserializeLookAroundAssertionKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'lookahead';
    case 1:
      return 'negativeLookahead';
    case 2:
      return 'lookbehind';
    case 3:
      return 'negativeLookbehind';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for LookAroundAssertionKind`);
  }
}

function deserializeCharacterKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'controlLetter';
    case 1:
      return 'hexadecimalEscape';
    case 2:
      return 'identifier';
    case 3:
      return 'null';
    case 4:
      return 'octal1';
    case 5:
      return 'octal2';
    case 6:
      return 'octal3';
    case 7:
      return 'singleEscape';
    case 8:
      return 'symbol';
    case 9:
      return 'unicodeEscape';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for CharacterKind`);
  }
}

function deserializeCharacterClassEscapeKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'd';
    case 1:
      return 'negativeD';
    case 2:
      return 's';
    case 3:
      return 'negativeS';
    case 4:
      return 'w';
    case 5:
      return 'negativeW';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for CharacterClassEscapeKind`);
  }
}

function deserializeCharacterClassContentsKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'union';
    case 1:
      return 'intersection';
    case 2:
      return 'subtraction';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for CharacterClassContentsKind`);
  }
}

function deserializeCharacterClassContents(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxCharacterClassRange(pos + 8);
    case 1:
      return deserializeBoxCharacterClassEscape(pos + 8);
    case 2:
      return deserializeBoxUnicodePropertyEscape(pos + 8);
    case 3:
      return deserializeBoxCharacter(pos + 8);
    case 4:
      return deserializeBoxCharacterClass(pos + 8);
    case 5:
      return deserializeBoxClassStringDisjunction(pos + 8);
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for CharacterClassContents`);
  }
}

function deserializeErrorSeverity(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'Error';
    case 1:
      return 'Warning';
    case 2:
      return 'Advice';
    default:
      throw new Error(`Unexpected discriminant ${uint8[pos]} for ErrorSeverity`);
  }
}

function deserializeStr(pos) {
  const pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  if (len === 0) return '';

  pos = uint32[pos32];
  if (sourceIsAscii && pos < sourceLen) return sourceText.substr(pos, len);

  // Longer strings use `TextDecoder`
  // TODO: Find best switch-over point
  const end = pos + len;
  if (len > 50) return decodeStr(uint8.subarray(pos, end));

  // Shorter strings decode by hand to avoid native call
  let out = '',
    c;
  do {
    c = uint8[pos++];
    if (c < 0x80) {
      out += fromCodePoint(c);
    } else {
      out += decodeStr(uint8.subarray(pos - 1, end));
      break;
    }
  } while (pos < end);

  return out;
}

function deserializeVecComment(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeComment(pos));
    pos += 24;
  }
  return arr;
}

function deserializeOptionHashbang(pos) {
  if (uint32[(pos + 8) >> 2] === 0 && uint32[(pos + 12) >> 2] === 0) return null;
  return deserializeHashbang(pos);
}

function deserializeVecDirective(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeDirective(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecStatement(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeStatement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionScopeId(pos) {
  if (uint32[pos >> 2] === 0) return null;
  return deserializeScopeId(pos);
}

function deserializeBoxBooleanLiteral(pos) {
  return deserializeBooleanLiteral(uint32[pos >> 2]);
}

function deserializeBoxNullLiteral(pos) {
  return deserializeNullLiteral(uint32[pos >> 2]);
}

function deserializeBoxNumericLiteral(pos) {
  return deserializeNumericLiteral(uint32[pos >> 2]);
}

function deserializeBoxBigIntLiteral(pos) {
  return deserializeBigIntLiteral(uint32[pos >> 2]);
}

function deserializeBoxRegExpLiteral(pos) {
  return deserializeRegExpLiteral(uint32[pos >> 2]);
}

function deserializeBoxStringLiteral(pos) {
  return deserializeStringLiteral(uint32[pos >> 2]);
}

function deserializeBoxTemplateLiteral(pos) {
  return deserializeTemplateLiteral(uint32[pos >> 2]);
}

function deserializeBoxIdentifierReference(pos) {
  return deserializeIdentifierReference(uint32[pos >> 2]);
}

function deserializeBoxMetaProperty(pos) {
  return deserializeMetaProperty(uint32[pos >> 2]);
}

function deserializeBoxSuper(pos) {
  return deserializeSuper(uint32[pos >> 2]);
}

function deserializeBoxArrayExpression(pos) {
  return deserializeArrayExpression(uint32[pos >> 2]);
}

function deserializeBoxArrowFunctionExpression(pos) {
  return deserializeArrowFunctionExpression(uint32[pos >> 2]);
}

function deserializeBoxAssignmentExpression(pos) {
  return deserializeAssignmentExpression(uint32[pos >> 2]);
}

function deserializeBoxAwaitExpression(pos) {
  return deserializeAwaitExpression(uint32[pos >> 2]);
}

function deserializeBoxBinaryExpression(pos) {
  return deserializeBinaryExpression(uint32[pos >> 2]);
}

function deserializeBoxCallExpression(pos) {
  return deserializeCallExpression(uint32[pos >> 2]);
}

function deserializeBoxChainExpression(pos) {
  return deserializeChainExpression(uint32[pos >> 2]);
}

function deserializeBoxClass(pos) {
  return deserializeClass(uint32[pos >> 2]);
}

function deserializeBoxConditionalExpression(pos) {
  return deserializeConditionalExpression(uint32[pos >> 2]);
}

function deserializeBoxFunction(pos) {
  return deserializeFunction(uint32[pos >> 2]);
}

function deserializeBoxImportExpression(pos) {
  return deserializeImportExpression(uint32[pos >> 2]);
}

function deserializeBoxLogicalExpression(pos) {
  return deserializeLogicalExpression(uint32[pos >> 2]);
}

function deserializeBoxNewExpression(pos) {
  return deserializeNewExpression(uint32[pos >> 2]);
}

function deserializeBoxObjectExpression(pos) {
  return deserializeObjectExpression(uint32[pos >> 2]);
}

function deserializeBoxParenthesizedExpression(pos) {
  return deserializeParenthesizedExpression(uint32[pos >> 2]);
}

function deserializeBoxSequenceExpression(pos) {
  return deserializeSequenceExpression(uint32[pos >> 2]);
}

function deserializeBoxTaggedTemplateExpression(pos) {
  return deserializeTaggedTemplateExpression(uint32[pos >> 2]);
}

function deserializeBoxThisExpression(pos) {
  return deserializeThisExpression(uint32[pos >> 2]);
}

function deserializeBoxUnaryExpression(pos) {
  return deserializeUnaryExpression(uint32[pos >> 2]);
}

function deserializeBoxUpdateExpression(pos) {
  return deserializeUpdateExpression(uint32[pos >> 2]);
}

function deserializeBoxYieldExpression(pos) {
  return deserializeYieldExpression(uint32[pos >> 2]);
}

function deserializeBoxPrivateInExpression(pos) {
  return deserializePrivateInExpression(uint32[pos >> 2]);
}

function deserializeBoxJSXElement(pos) {
  return deserializeJSXElement(uint32[pos >> 2]);
}

function deserializeBoxJSXFragment(pos) {
  return deserializeJSXFragment(uint32[pos >> 2]);
}

function deserializeBoxTSAsExpression(pos) {
  return deserializeTSAsExpression(uint32[pos >> 2]);
}

function deserializeBoxTSSatisfiesExpression(pos) {
  return deserializeTSSatisfiesExpression(uint32[pos >> 2]);
}

function deserializeBoxTSTypeAssertion(pos) {
  return deserializeTSTypeAssertion(uint32[pos >> 2]);
}

function deserializeBoxTSNonNullExpression(pos) {
  return deserializeTSNonNullExpression(uint32[pos >> 2]);
}

function deserializeBoxTSInstantiationExpression(pos) {
  return deserializeTSInstantiationExpression(uint32[pos >> 2]);
}

function deserializeBoxV8IntrinsicExpression(pos) {
  return deserializeV8IntrinsicExpression(uint32[pos >> 2]);
}

function deserializeOptionReferenceId(pos) {
  if (uint32[pos >> 2] === 0) return null;
  return deserializeReferenceId(pos);
}

function deserializeOptionSymbolId(pos) {
  if (uint32[pos >> 2] === 0) return null;
  return deserializeSymbolId(pos);
}

function deserializeVecArrayExpressionElement(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeArrayExpressionElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionSpan(pos) {
  if (uint8[pos] === 0) return null;
  return deserializeSpan(pos + 8);
}

function deserializeBoxSpreadElement(pos) {
  return deserializeSpreadElement(uint32[pos >> 2]);
}

function deserializeVecObjectPropertyKind(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeObjectPropertyKind(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxObjectProperty(pos) {
  return deserializeObjectProperty(uint32[pos >> 2]);
}

function deserializeBool(pos) {
  return uint8[pos] === 1;
}

function deserializeBoxIdentifierName(pos) {
  return deserializeIdentifierName(uint32[pos >> 2]);
}

function deserializeBoxPrivateIdentifier(pos) {
  return deserializePrivateIdentifier(uint32[pos >> 2]);
}

function deserializeVecTemplateElement(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTemplateElement(pos));
    pos += 48;
  }
  return arr;
}

function deserializeVecExpression(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeExpression(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSTypeParameterInstantiation(pos) {
  return deserializeTSTypeParameterInstantiation(uint32[pos >> 2]);
}

function deserializeOptionBoxTSTypeParameterInstantiation(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxTSTypeParameterInstantiation(pos);
}

function deserializeOptionStr(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeStr(pos);
}

function deserializeBoxComputedMemberExpression(pos) {
  return deserializeComputedMemberExpression(uint32[pos >> 2]);
}

function deserializeBoxStaticMemberExpression(pos) {
  return deserializeStaticMemberExpression(uint32[pos >> 2]);
}

function deserializeBoxPrivateFieldExpression(pos) {
  return deserializePrivateFieldExpression(uint32[pos >> 2]);
}

function deserializeVecArgument(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeArgument(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxArrayAssignmentTarget(pos) {
  return deserializeArrayAssignmentTarget(uint32[pos >> 2]);
}

function deserializeBoxObjectAssignmentTarget(pos) {
  return deserializeObjectAssignmentTarget(uint32[pos >> 2]);
}

function deserializeOptionAssignmentTargetMaybeDefault(pos) {
  if (uint8[pos] === 51) return null;
  return deserializeAssignmentTargetMaybeDefault(pos);
}

function deserializeVecOptionAssignmentTargetMaybeDefault(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeOptionAssignmentTargetMaybeDefault(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionAssignmentTargetRest(pos) {
  if (uint8[pos + 8] === 51) return null;
  return deserializeAssignmentTargetRest(pos);
}

function deserializeVecAssignmentTargetProperty(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeAssignmentTargetProperty(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxAssignmentTargetWithDefault(pos) {
  return deserializeAssignmentTargetWithDefault(uint32[pos >> 2]);
}

function deserializeBoxAssignmentTargetPropertyIdentifier(pos) {
  return deserializeAssignmentTargetPropertyIdentifier(uint32[pos >> 2]);
}

function deserializeBoxAssignmentTargetPropertyProperty(pos) {
  return deserializeAssignmentTargetPropertyProperty(uint32[pos >> 2]);
}

function deserializeOptionExpression(pos) {
  if (uint8[pos] === 51) return null;
  return deserializeExpression(pos);
}

function deserializeBoxBlockStatement(pos) {
  return deserializeBlockStatement(uint32[pos >> 2]);
}

function deserializeBoxBreakStatement(pos) {
  return deserializeBreakStatement(uint32[pos >> 2]);
}

function deserializeBoxContinueStatement(pos) {
  return deserializeContinueStatement(uint32[pos >> 2]);
}

function deserializeBoxDebuggerStatement(pos) {
  return deserializeDebuggerStatement(uint32[pos >> 2]);
}

function deserializeBoxDoWhileStatement(pos) {
  return deserializeDoWhileStatement(uint32[pos >> 2]);
}

function deserializeBoxEmptyStatement(pos) {
  return deserializeEmptyStatement(uint32[pos >> 2]);
}

function deserializeBoxExpressionStatement(pos) {
  return deserializeExpressionStatement(uint32[pos >> 2]);
}

function deserializeBoxForInStatement(pos) {
  return deserializeForInStatement(uint32[pos >> 2]);
}

function deserializeBoxForOfStatement(pos) {
  return deserializeForOfStatement(uint32[pos >> 2]);
}

function deserializeBoxForStatement(pos) {
  return deserializeForStatement(uint32[pos >> 2]);
}

function deserializeBoxIfStatement(pos) {
  return deserializeIfStatement(uint32[pos >> 2]);
}

function deserializeBoxLabeledStatement(pos) {
  return deserializeLabeledStatement(uint32[pos >> 2]);
}

function deserializeBoxReturnStatement(pos) {
  return deserializeReturnStatement(uint32[pos >> 2]);
}

function deserializeBoxSwitchStatement(pos) {
  return deserializeSwitchStatement(uint32[pos >> 2]);
}

function deserializeBoxThrowStatement(pos) {
  return deserializeThrowStatement(uint32[pos >> 2]);
}

function deserializeBoxTryStatement(pos) {
  return deserializeTryStatement(uint32[pos >> 2]);
}

function deserializeBoxWhileStatement(pos) {
  return deserializeWhileStatement(uint32[pos >> 2]);
}

function deserializeBoxWithStatement(pos) {
  return deserializeWithStatement(uint32[pos >> 2]);
}

function deserializeBoxVariableDeclaration(pos) {
  return deserializeVariableDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSTypeAliasDeclaration(pos) {
  return deserializeTSTypeAliasDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSInterfaceDeclaration(pos) {
  return deserializeTSInterfaceDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSEnumDeclaration(pos) {
  return deserializeTSEnumDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSModuleDeclaration(pos) {
  return deserializeTSModuleDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSImportEqualsDeclaration(pos) {
  return deserializeTSImportEqualsDeclaration(uint32[pos >> 2]);
}

function deserializeVecVariableDeclarator(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeVariableDeclarator(pos));
    pos += 72;
  }
  return arr;
}

function deserializeOptionStatement(pos) {
  if (uint8[pos] === 70) return null;
  return deserializeStatement(pos);
}

function deserializeOptionForStatementInit(pos) {
  if (uint8[pos] === 65) return null;
  return deserializeForStatementInit(pos);
}

function deserializeOptionLabelIdentifier(pos) {
  if (uint32[(pos + 8) >> 2] === 0 && uint32[(pos + 12) >> 2] === 0) return null;
  return deserializeLabelIdentifier(pos);
}

function deserializeVecSwitchCase(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeSwitchCase(pos));
    pos += 56;
  }
  return arr;
}

function deserializeBoxCatchClause(pos) {
  return deserializeCatchClause(uint32[pos >> 2]);
}

function deserializeOptionBoxCatchClause(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxCatchClause(pos);
}

function deserializeOptionBoxBlockStatement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxBlockStatement(pos);
}

function deserializeOptionCatchParameter(pos) {
  if (uint8[pos + 32] === 2) return null;
  return deserializeCatchParameter(pos);
}

function deserializeBoxTSTypeAnnotation(pos) {
  return deserializeTSTypeAnnotation(uint32[pos >> 2]);
}

function deserializeOptionBoxTSTypeAnnotation(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxTSTypeAnnotation(pos);
}

function deserializeBoxBindingIdentifier(pos) {
  return deserializeBindingIdentifier(uint32[pos >> 2]);
}

function deserializeBoxObjectPattern(pos) {
  return deserializeObjectPattern(uint32[pos >> 2]);
}

function deserializeBoxArrayPattern(pos) {
  return deserializeArrayPattern(uint32[pos >> 2]);
}

function deserializeBoxAssignmentPattern(pos) {
  return deserializeAssignmentPattern(uint32[pos >> 2]);
}

function deserializeVecBindingProperty(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeBindingProperty(pos));
    pos += 64;
  }
  return arr;
}

function deserializeBoxBindingRestElement(pos) {
  return deserializeBindingRestElement(uint32[pos >> 2]);
}

function deserializeOptionBoxBindingRestElement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxBindingRestElement(pos);
}

function deserializeOptionBindingPattern(pos) {
  if (uint8[pos + 24] === 2) return null;
  return deserializeBindingPattern(pos);
}

function deserializeVecOptionBindingPattern(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeOptionBindingPattern(pos));
    pos += 32;
  }
  return arr;
}

function deserializeOptionBindingIdentifier(pos) {
  if (uint32[(pos + 8) >> 2] === 0 && uint32[(pos + 12) >> 2] === 0) return null;
  return deserializeBindingIdentifier(pos);
}

function deserializeBoxTSTypeParameterDeclaration(pos) {
  return deserializeTSTypeParameterDeclaration(uint32[pos >> 2]);
}

function deserializeOptionBoxTSTypeParameterDeclaration(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxTSTypeParameterDeclaration(pos);
}

function deserializeBoxTSThisParameter(pos) {
  return deserializeTSThisParameter(uint32[pos >> 2]);
}

function deserializeOptionBoxTSThisParameter(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxTSThisParameter(pos);
}

function deserializeBoxFormalParameters(pos) {
  return deserializeFormalParameters(uint32[pos >> 2]);
}

function deserializeBoxFunctionBody(pos) {
  return deserializeFunctionBody(uint32[pos >> 2]);
}

function deserializeOptionBoxFunctionBody(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxFunctionBody(pos);
}

function deserializeVecFormalParameter(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeFormalParameter(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecDecorator(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeDecorator(pos));
    pos += 24;
  }
  return arr;
}

function deserializeOptionTSAccessibility(pos) {
  if (uint8[pos] === 3) return null;
  return deserializeTSAccessibility(pos);
}

function deserializeVecTSClassImplements(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSClassImplements(pos));
    pos += 32;
  }
  return arr;
}

function deserializeOptionVecTSClassImplements(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeVecTSClassImplements(pos);
}

function deserializeBoxClassBody(pos) {
  return deserializeClassBody(uint32[pos >> 2]);
}

function deserializeVecClassElement(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeClassElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxStaticBlock(pos) {
  return deserializeStaticBlock(uint32[pos >> 2]);
}

function deserializeBoxMethodDefinition(pos) {
  return deserializeMethodDefinition(uint32[pos >> 2]);
}

function deserializeBoxPropertyDefinition(pos) {
  return deserializePropertyDefinition(uint32[pos >> 2]);
}

function deserializeBoxAccessorProperty(pos) {
  return deserializeAccessorProperty(uint32[pos >> 2]);
}

function deserializeBoxTSIndexSignature(pos) {
  return deserializeTSIndexSignature(uint32[pos >> 2]);
}

function deserializeBoxImportDeclaration(pos) {
  return deserializeImportDeclaration(uint32[pos >> 2]);
}

function deserializeBoxExportAllDeclaration(pos) {
  return deserializeExportAllDeclaration(uint32[pos >> 2]);
}

function deserializeBoxExportDefaultDeclaration(pos) {
  return deserializeExportDefaultDeclaration(uint32[pos >> 2]);
}

function deserializeBoxExportNamedDeclaration(pos) {
  return deserializeExportNamedDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSExportAssignment(pos) {
  return deserializeTSExportAssignment(uint32[pos >> 2]);
}

function deserializeBoxTSNamespaceExportDeclaration(pos) {
  return deserializeTSNamespaceExportDeclaration(uint32[pos >> 2]);
}

function deserializeOptionImportPhase(pos) {
  if (uint8[pos] === 2) return null;
  return deserializeImportPhase(pos);
}

function deserializeVecImportDeclarationSpecifier(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeImportDeclarationSpecifier(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionVecImportDeclarationSpecifier(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeVecImportDeclarationSpecifier(pos);
}

function deserializeBoxWithClause(pos) {
  return deserializeWithClause(uint32[pos >> 2]);
}

function deserializeOptionBoxWithClause(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxWithClause(pos);
}

function deserializeBoxImportSpecifier(pos) {
  return deserializeImportSpecifier(uint32[pos >> 2]);
}

function deserializeBoxImportDefaultSpecifier(pos) {
  return deserializeImportDefaultSpecifier(uint32[pos >> 2]);
}

function deserializeBoxImportNamespaceSpecifier(pos) {
  return deserializeImportNamespaceSpecifier(uint32[pos >> 2]);
}

function deserializeVecImportAttribute(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeImportAttribute(pos));
    pos += 112;
  }
  return arr;
}

function deserializeOptionDeclaration(pos) {
  if (uint8[pos] === 31) return null;
  return deserializeDeclaration(pos);
}

function deserializeVecExportSpecifier(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeExportSpecifier(pos));
    pos += 128;
  }
  return arr;
}

function deserializeOptionStringLiteral(pos) {
  if (uint8[pos + 40] === 2) return null;
  return deserializeStringLiteral(pos);
}

function deserializeOptionModuleExportName(pos) {
  if (uint8[pos] === 3) return null;
  return deserializeModuleExportName(pos);
}

function deserializeF64(pos) {
  return float64[pos >> 3];
}

function deserializeBoxPattern(pos) {
  return deserializePattern(uint32[pos >> 2]);
}

function deserializeU8(pos) {
  return uint8[pos];
}

function deserializeBoxJSXOpeningElement(pos) {
  return deserializeJSXOpeningElement(uint32[pos >> 2]);
}

function deserializeBoxJSXClosingElement(pos) {
  return deserializeJSXClosingElement(uint32[pos >> 2]);
}

function deserializeOptionBoxJSXClosingElement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxJSXClosingElement(pos);
}

function deserializeVecJSXChild(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeJSXChild(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecJSXAttributeItem(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeJSXAttributeItem(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxJSXIdentifier(pos) {
  return deserializeJSXIdentifier(uint32[pos >> 2]);
}

function deserializeBoxJSXNamespacedName(pos) {
  return deserializeJSXNamespacedName(uint32[pos >> 2]);
}

function deserializeBoxJSXMemberExpression(pos) {
  return deserializeJSXMemberExpression(uint32[pos >> 2]);
}

function deserializeBoxJSXAttribute(pos) {
  return deserializeJSXAttribute(uint32[pos >> 2]);
}

function deserializeBoxJSXSpreadAttribute(pos) {
  return deserializeJSXSpreadAttribute(uint32[pos >> 2]);
}

function deserializeOptionJSXAttributeValue(pos) {
  if (uint8[pos] === 4) return null;
  return deserializeJSXAttributeValue(pos);
}

function deserializeBoxJSXExpressionContainer(pos) {
  return deserializeJSXExpressionContainer(uint32[pos >> 2]);
}

function deserializeBoxJSXText(pos) {
  return deserializeJSXText(uint32[pos >> 2]);
}

function deserializeBoxJSXSpreadChild(pos) {
  return deserializeJSXSpreadChild(uint32[pos >> 2]);
}

function deserializeVecTSEnumMember(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSEnumMember(pos));
    pos += 40;
  }
  return arr;
}

function deserializeBoxTSAnyKeyword(pos) {
  return deserializeTSAnyKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSBigIntKeyword(pos) {
  return deserializeTSBigIntKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSBooleanKeyword(pos) {
  return deserializeTSBooleanKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSIntrinsicKeyword(pos) {
  return deserializeTSIntrinsicKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSNeverKeyword(pos) {
  return deserializeTSNeverKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSNullKeyword(pos) {
  return deserializeTSNullKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSNumberKeyword(pos) {
  return deserializeTSNumberKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSObjectKeyword(pos) {
  return deserializeTSObjectKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSStringKeyword(pos) {
  return deserializeTSStringKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSSymbolKeyword(pos) {
  return deserializeTSSymbolKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSUndefinedKeyword(pos) {
  return deserializeTSUndefinedKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSUnknownKeyword(pos) {
  return deserializeTSUnknownKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSVoidKeyword(pos) {
  return deserializeTSVoidKeyword(uint32[pos >> 2]);
}

function deserializeBoxTSArrayType(pos) {
  return deserializeTSArrayType(uint32[pos >> 2]);
}

function deserializeBoxTSConditionalType(pos) {
  return deserializeTSConditionalType(uint32[pos >> 2]);
}

function deserializeBoxTSConstructorType(pos) {
  return deserializeTSConstructorType(uint32[pos >> 2]);
}

function deserializeBoxTSFunctionType(pos) {
  return deserializeTSFunctionType(uint32[pos >> 2]);
}

function deserializeBoxTSImportType(pos) {
  return deserializeTSImportType(uint32[pos >> 2]);
}

function deserializeBoxTSIndexedAccessType(pos) {
  return deserializeTSIndexedAccessType(uint32[pos >> 2]);
}

function deserializeBoxTSInferType(pos) {
  return deserializeTSInferType(uint32[pos >> 2]);
}

function deserializeBoxTSIntersectionType(pos) {
  return deserializeTSIntersectionType(uint32[pos >> 2]);
}

function deserializeBoxTSLiteralType(pos) {
  return deserializeTSLiteralType(uint32[pos >> 2]);
}

function deserializeBoxTSMappedType(pos) {
  return deserializeTSMappedType(uint32[pos >> 2]);
}

function deserializeBoxTSNamedTupleMember(pos) {
  return deserializeTSNamedTupleMember(uint32[pos >> 2]);
}

function deserializeBoxTSTemplateLiteralType(pos) {
  return deserializeTSTemplateLiteralType(uint32[pos >> 2]);
}

function deserializeBoxTSThisType(pos) {
  return deserializeTSThisType(uint32[pos >> 2]);
}

function deserializeBoxTSTupleType(pos) {
  return deserializeTSTupleType(uint32[pos >> 2]);
}

function deserializeBoxTSTypeLiteral(pos) {
  return deserializeTSTypeLiteral(uint32[pos >> 2]);
}

function deserializeBoxTSTypeOperator(pos) {
  return deserializeTSTypeOperator(uint32[pos >> 2]);
}

function deserializeBoxTSTypePredicate(pos) {
  return deserializeTSTypePredicate(uint32[pos >> 2]);
}

function deserializeBoxTSTypeQuery(pos) {
  return deserializeTSTypeQuery(uint32[pos >> 2]);
}

function deserializeBoxTSTypeReference(pos) {
  return deserializeTSTypeReference(uint32[pos >> 2]);
}

function deserializeBoxTSUnionType(pos) {
  return deserializeTSUnionType(uint32[pos >> 2]);
}

function deserializeBoxTSParenthesizedType(pos) {
  return deserializeTSParenthesizedType(uint32[pos >> 2]);
}

function deserializeBoxJSDocNullableType(pos) {
  return deserializeJSDocNullableType(uint32[pos >> 2]);
}

function deserializeBoxJSDocNonNullableType(pos) {
  return deserializeJSDocNonNullableType(uint32[pos >> 2]);
}

function deserializeBoxJSDocUnknownType(pos) {
  return deserializeJSDocUnknownType(uint32[pos >> 2]);
}

function deserializeVecTSType(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSType(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecTSTupleElement(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSTupleElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSOptionalType(pos) {
  return deserializeTSOptionalType(uint32[pos >> 2]);
}

function deserializeBoxTSRestType(pos) {
  return deserializeTSRestType(uint32[pos >> 2]);
}

function deserializeBoxTSQualifiedName(pos) {
  return deserializeTSQualifiedName(uint32[pos >> 2]);
}

function deserializeOptionTSType(pos) {
  if (uint8[pos] === 38) return null;
  return deserializeTSType(pos);
}

function deserializeVecTSTypeParameter(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSTypeParameter(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecTSInterfaceHeritage(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSInterfaceHeritage(pos));
    pos += 32;
  }
  return arr;
}

function deserializeOptionVecTSInterfaceHeritage(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeVecTSInterfaceHeritage(pos);
}

function deserializeBoxTSInterfaceBody(pos) {
  return deserializeTSInterfaceBody(uint32[pos >> 2]);
}

function deserializeVecTSSignature(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSSignature(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSPropertySignature(pos) {
  return deserializeTSPropertySignature(uint32[pos >> 2]);
}

function deserializeBoxTSCallSignatureDeclaration(pos) {
  return deserializeTSCallSignatureDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSConstructSignatureDeclaration(pos) {
  return deserializeTSConstructSignatureDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSMethodSignature(pos) {
  return deserializeTSMethodSignature(uint32[pos >> 2]);
}

function deserializeVecTSIndexSignatureName(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTSIndexSignatureName(pos));
    pos += 32;
  }
  return arr;
}

function deserializeOptionTSThisParameter(pos) {
  if (uint8[pos] === 0) return null;
  return deserializeTSThisParameter(pos + 8);
}

function deserializeOptionTSModuleDeclarationBody(pos) {
  if (uint8[pos] === 2) return null;
  return deserializeTSModuleDeclarationBody(pos);
}

function deserializeBoxTSModuleBlock(pos) {
  return deserializeTSModuleBlock(uint32[pos >> 2]);
}

function deserializeBoxTSTypeParameter(pos) {
  return deserializeTSTypeParameter(uint32[pos >> 2]);
}

function deserializeOptionBoxObjectExpression(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxObjectExpression(pos);
}

function deserializeOptionTSTypeName(pos) {
  if (uint8[pos] === 2) return null;
  return deserializeTSTypeName(pos);
}

function deserializeBoxTSExternalModuleReference(pos) {
  return deserializeTSExternalModuleReference(uint32[pos >> 2]);
}

function deserializeU32(pos) {
  return uint32[pos >> 2];
}

function deserializeOptionNameSpan(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeNameSpan(pos);
}

function deserializeVecAlternative(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeAlternative(pos));
    pos += 40;
  }
  return arr;
}

function deserializeVecTerm(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeTerm(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxBoundaryAssertion(pos) {
  return deserializeBoundaryAssertion(uint32[pos >> 2]);
}

function deserializeBoxLookAroundAssertion(pos) {
  return deserializeLookAroundAssertion(uint32[pos >> 2]);
}

function deserializeBoxQuantifier(pos) {
  return deserializeQuantifier(uint32[pos >> 2]);
}

function deserializeBoxCharacter(pos) {
  return deserializeCharacter(uint32[pos >> 2]);
}

function deserializeBoxCharacterClassEscape(pos) {
  return deserializeCharacterClassEscape(uint32[pos >> 2]);
}

function deserializeBoxUnicodePropertyEscape(pos) {
  return deserializeUnicodePropertyEscape(uint32[pos >> 2]);
}

function deserializeBoxCharacterClass(pos) {
  return deserializeCharacterClass(uint32[pos >> 2]);
}

function deserializeBoxCapturingGroup(pos) {
  return deserializeCapturingGroup(uint32[pos >> 2]);
}

function deserializeBoxIgnoreGroup(pos) {
  return deserializeIgnoreGroup(uint32[pos >> 2]);
}

function deserializeBoxIndexedReference(pos) {
  return deserializeIndexedReference(uint32[pos >> 2]);
}

function deserializeBoxNamedReference(pos) {
  return deserializeNamedReference(uint32[pos >> 2]);
}

function deserializeU64(pos) {
  const pos32 = pos >> 2;
  return uint32[pos32] + uint32[pos32 + 1] * 4294967296;
}

function deserializeOptionU64(pos) {
  if (uint8[pos] === 0) return null;
  return deserializeU64(pos + 8);
}

function deserializeVecCharacterClassContents(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeCharacterClassContents(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxCharacterClassRange(pos) {
  return deserializeCharacterClassRange(uint32[pos >> 2]);
}

function deserializeBoxClassStringDisjunction(pos) {
  return deserializeClassStringDisjunction(uint32[pos >> 2]);
}

function deserializeVecClassString(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeClassString(pos));
    pos += 48;
  }
  return arr;
}

function deserializeVecCharacter(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeCharacter(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionModifiers(pos) {
  if (uint8[pos + 8] === 3) return null;
  return deserializeModifiers(pos);
}

function deserializeOptionModifier(pos) {
  if (uint8[pos] === 2) return null;
  return deserializeModifier(pos);
}

function deserializeVecError(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeError(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecErrorLabel(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeErrorLabel(pos));
    pos += 24;
  }
  return arr;
}

function deserializeVecStaticImport(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeStaticImport(pos));
    pos += 64;
  }
  return arr;
}

function deserializeVecStaticExport(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeStaticExport(pos));
    pos += 40;
  }
  return arr;
}

function deserializeVecDynamicImport(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeDynamicImport(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecSpan(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeSpan(pos));
    pos += 8;
  }
  return arr;
}

function deserializeVecImportEntry(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeImportEntry(pos));
    pos += 96;
  }
  return arr;
}

function deserializeVecExportEntry(pos) {
  const arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 6];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(deserializeExportEntry(pos));
    pos += 144;
  }
  return arr;
}
