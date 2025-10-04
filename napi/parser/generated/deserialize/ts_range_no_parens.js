// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8, uint32, float64, sourceText, sourceIsAscii, sourceByteLen;
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true }),
  decodeStr = textDecoder.decode.bind(textDecoder),
  { fromCodePoint } = String;

export function deserialize(buffer, sourceText, sourceByteLen) {
  return deserializeWith(buffer, sourceText, sourceByteLen, deserializeRawTransferData);
}

export function deserializeProgramOnly(buffer, sourceText, sourceByteLen) {
  return deserializeWith(buffer, sourceText, sourceByteLen, deserializeProgram);
}

function deserializeWith(buffer, sourceTextInput, sourceByteLenInput, deserialize) {
  uint8 = buffer;
  uint32 = buffer.uint32;
  float64 = buffer.float64;
  sourceText = sourceTextInput;
  sourceByteLen = sourceByteLenInput;
  sourceIsAscii = sourceText.length === sourceByteLen;
  let data = deserialize(uint32[536870902]);
  uint8 =
    uint32 =
    float64 =
    sourceText =
      void 0;
  return data;
}

function deserializeProgram(pos) {
  let body = deserializeVecDirective(pos + 72);
  body.push(...deserializeVecStatement(pos + 96));
  let end = deserializeU32(pos + 4), start;
  if (body.length > 0) {
    let first = body[0];
    start = first.start;
    if (first.type === 'ExportNamedDeclaration' || first.type === 'ExportDefaultDeclaration') {
      let { declaration } = first;
      if (declaration !== null && declaration.type === 'ClassDeclaration' && declaration.decorators.length > 0) {
        let decoratorStart = declaration.decorators[0].start;
        decoratorStart < start && (start = decoratorStart);
      }
    }
  } else start = end;
  return {
    type: 'Program',
    body,
    sourceType: deserializeModuleKind(pos + 125),
    hashbang: deserializeOptionHashbang(pos + 48),
    start,
    end,
    range: [start, end],
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for Expression`);
  }
}

function deserializeIdentifierName(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Identifier',
    decorators: [],
    name: deserializeStr(pos + 8),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeIdentifierReference(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Identifier',
    decorators: [],
    name: deserializeStr(pos + 8),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeBindingIdentifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Identifier',
    decorators: [],
    name: deserializeStr(pos + 8),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeLabelIdentifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Identifier',
    decorators: [],
    name: deserializeStr(pos + 8),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeThisExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ThisExpression',
    start,
    end,
    range: [start, end],
  };
}

function deserializeArrayExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ArrayExpression',
    elements: deserializeVecArrayExpressionElement(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ArrayExpressionElement`);
  }
}

function deserializeElision(pos) {
  return null;
}

function deserializeObjectExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ObjectExpression',
    properties: deserializeVecObjectPropertyKind(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeObjectPropertyKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxObjectProperty(pos + 8);
    case 1:
      return deserializeBoxSpreadElement(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ObjectPropertyKind`);
  }
}

function deserializeObjectProperty(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Property',
    kind: deserializePropertyKind(pos + 40),
    key: deserializePropertyKey(pos + 8),
    value: deserializeExpression(pos + 24),
    method: deserializeBool(pos + 41),
    shorthand: deserializeBool(pos + 42),
    computed: deserializeBool(pos + 43),
    optional: false,
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for PropertyKey`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for PropertyKind`);
  }
}

function deserializeTemplateLiteral(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TemplateLiteral',
    quasis: deserializeVecTemplateElement(pos + 8),
    expressions: deserializeVecExpression(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTaggedTemplateExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TaggedTemplateExpression',
    tag: deserializeExpression(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    quasi: deserializeTemplateLiteral(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTemplateElement(pos) {
  let tail = deserializeBool(pos + 40),
    start = deserializeU32(pos) - 1,
    end = deserializeU32(pos + 4) + 2 - tail,
    value = deserializeTemplateElementValue(pos + 8);
  value.cooked !== null && deserializeBool(pos + 41) &&
    (value.cooked = value.cooked.replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16))));
  return {
    type: 'TemplateElement',
    value,
    tail,
    start,
    end,
    range: [start, end],
  };
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'MemberExpression',
    object: deserializeExpression(pos + 8),
    property: deserializeExpression(pos + 24),
    optional: deserializeBool(pos + 40),
    computed: true,
    start,
    end,
    range: [start, end],
  };
}

function deserializeStaticMemberExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'MemberExpression',
    object: deserializeExpression(pos + 8),
    property: deserializeIdentifierName(pos + 24),
    optional: deserializeBool(pos + 48),
    computed: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializePrivateFieldExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'MemberExpression',
    object: deserializeExpression(pos + 8),
    property: deserializePrivateIdentifier(pos + 24),
    optional: deserializeBool(pos + 48),
    computed: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeCallExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'CallExpression',
    callee: deserializeExpression(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    arguments: deserializeVecArgument(pos + 32),
    optional: deserializeBool(pos + 56),
    start,
    end,
    range: [start, end],
  };
}

function deserializeNewExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'NewExpression',
    callee: deserializeExpression(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    arguments: deserializeVecArgument(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeMetaProperty(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'MetaProperty',
    meta: deserializeIdentifierName(pos + 8),
    property: deserializeIdentifierName(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeSpreadElement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'SpreadElement',
    argument: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for Argument`);
  }
}

function deserializeUpdateExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'UpdateExpression',
    operator: deserializeUpdateOperator(pos + 24),
    prefix: deserializeBool(pos + 25),
    argument: deserializeSimpleAssignmentTarget(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeUnaryExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'UnaryExpression',
    operator: deserializeUnaryOperator(pos + 24),
    argument: deserializeExpression(pos + 8),
    prefix: true,
    start,
    end,
    range: [start, end],
  };
}

function deserializeBinaryExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'BinaryExpression',
    left: deserializeExpression(pos + 8),
    operator: deserializeBinaryOperator(pos + 40),
    right: deserializeExpression(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializePrivateInExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'BinaryExpression',
    left: deserializePrivateIdentifier(pos + 8),
    operator: 'in',
    right: deserializeExpression(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeLogicalExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'LogicalExpression',
    left: deserializeExpression(pos + 8),
    operator: deserializeLogicalOperator(pos + 40),
    right: deserializeExpression(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeConditionalExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ConditionalExpression',
    test: deserializeExpression(pos + 8),
    consequent: deserializeExpression(pos + 24),
    alternate: deserializeExpression(pos + 40),
    start,
    end,
    range: [start, end],
  };
}

function deserializeAssignmentExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'AssignmentExpression',
    operator: deserializeAssignmentOperator(pos + 40),
    left: deserializeAssignmentTarget(pos + 8),
    right: deserializeExpression(pos + 24),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTarget`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for SimpleAssignmentTarget`);
  }
}

function deserializeArrayAssignmentTarget(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 8),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 32);
  rest !== null && elements.push(rest);
  return {
    type: 'ArrayPattern',
    decorators: [],
    elements,
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeObjectAssignmentTarget(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    properties = deserializeVecAssignmentTargetProperty(pos + 8),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 32);
  rest !== null && properties.push(rest);
  return {
    type: 'ObjectPattern',
    decorators: [],
    properties,
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeAssignmentTargetRest(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'RestElement',
    decorators: [],
    argument: deserializeAssignmentTarget(pos + 8),
    optional: false,
    typeAnnotation: null,
    value: null,
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetMaybeDefault`);
  }
}

function deserializeAssignmentTargetWithDefault(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'AssignmentPattern',
    decorators: [],
    left: deserializeAssignmentTarget(pos + 8),
    right: deserializeExpression(pos + 24),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeAssignmentTargetProperty(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxAssignmentTargetPropertyIdentifier(pos + 8);
    case 1:
      return deserializeBoxAssignmentTargetPropertyProperty(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetProperty`);
  }
}

function deserializeAssignmentTargetPropertyIdentifier(pos) {
  let key = deserializeIdentifierReference(pos + 8),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    init = deserializeOptionExpression(pos + 40),
    keyCopy = { ...key };
  return {
    type: 'Property',
    kind: 'init',
    key,
    value: init === null ? keyCopy : {
      type: 'AssignmentPattern',
      decorators: [],
      left: keyCopy,
      right: init,
      optional: false,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
    },
    method: false,
    shorthand: true,
    computed: false,
    optional: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Property',
    kind: 'init',
    key: deserializePropertyKey(pos + 8),
    value: deserializeAssignmentTargetMaybeDefault(pos + 24),
    method: false,
    shorthand: false,
    computed: deserializeBool(pos + 40),
    optional: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeSequenceExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'SequenceExpression',
    expressions: deserializeVecExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeSuper(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Super',
    start,
    end,
    range: [start, end],
  };
}

function deserializeAwaitExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'AwaitExpression',
    argument: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeChainExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ChainExpression',
    expression: deserializeChainElement(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ChainElement`);
  }
}

function deserializeParenthesizedExpression(pos) {
  return deserializeExpression(pos + 8);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for Statement`);
  }
}

function deserializeDirective(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ExpressionStatement',
    expression: deserializeStringLiteral(pos + 8),
    directive: deserializeStr(pos + 56),
    start,
    end,
    range: [start, end],
  };
}

function deserializeHashbang(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Hashbang',
    value: deserializeStr(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeBlockStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'BlockStatement',
    body: deserializeVecStatement(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for Declaration`);
  }
}

function deserializeVariableDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'VariableDeclaration',
    kind: deserializeVariableDeclarationKind(pos + 32),
    declarations: deserializeVecVariableDeclarator(pos + 8),
    declare: deserializeBool(pos + 33),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for VariableDeclarationKind`);
  }
}

function deserializeVariableDeclarator(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'VariableDeclarator',
    id: deserializeBindingPattern(pos + 8),
    init: deserializeOptionExpression(pos + 40),
    definite: deserializeBool(pos + 57),
    start,
    end,
    range: [start, end],
  };
}

function deserializeEmptyStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'EmptyStatement',
    start,
    end,
    range: [start, end],
  };
}

function deserializeExpressionStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ExpressionStatement',
    expression: deserializeExpression(pos + 8),
    directive: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeIfStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'IfStatement',
    test: deserializeExpression(pos + 8),
    consequent: deserializeStatement(pos + 24),
    alternate: deserializeOptionStatement(pos + 40),
    start,
    end,
    range: [start, end],
  };
}

function deserializeDoWhileStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'DoWhileStatement',
    body: deserializeStatement(pos + 8),
    test: deserializeExpression(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeWhileStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'WhileStatement',
    test: deserializeExpression(pos + 8),
    body: deserializeStatement(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeForStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ForStatement',
    init: deserializeOptionForStatementInit(pos + 8),
    test: deserializeOptionExpression(pos + 24),
    update: deserializeOptionExpression(pos + 40),
    body: deserializeStatement(pos + 56),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ForStatementInit`);
  }
}

function deserializeForInStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ForInStatement',
    left: deserializeForStatementLeft(pos + 8),
    right: deserializeExpression(pos + 24),
    body: deserializeStatement(pos + 40),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ForStatementLeft`);
  }
}

function deserializeForOfStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ForOfStatement',
    await: deserializeBool(pos + 60),
    left: deserializeForStatementLeft(pos + 8),
    right: deserializeExpression(pos + 24),
    body: deserializeStatement(pos + 40),
    start,
    end,
    range: [start, end],
  };
}

function deserializeContinueStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ContinueStatement',
    label: deserializeOptionLabelIdentifier(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeBreakStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'BreakStatement',
    label: deserializeOptionLabelIdentifier(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeReturnStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ReturnStatement',
    argument: deserializeOptionExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeWithStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'WithStatement',
    object: deserializeExpression(pos + 8),
    body: deserializeStatement(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeSwitchStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'SwitchStatement',
    discriminant: deserializeExpression(pos + 8),
    cases: deserializeVecSwitchCase(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeSwitchCase(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'SwitchCase',
    test: deserializeOptionExpression(pos + 8),
    consequent: deserializeVecStatement(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeLabeledStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'LabeledStatement',
    label: deserializeLabelIdentifier(pos + 8),
    body: deserializeStatement(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeThrowStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ThrowStatement',
    argument: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTryStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TryStatement',
    block: deserializeBoxBlockStatement(pos + 8),
    handler: deserializeOptionBoxCatchClause(pos + 16),
    finalizer: deserializeOptionBoxBlockStatement(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeCatchClause(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'CatchClause',
    param: deserializeOptionCatchParameter(pos + 8),
    body: deserializeBoxBlockStatement(pos + 48),
    start,
    end,
    range: [start, end],
  };
}

function deserializeCatchParameter(pos) {
  return deserializeBindingPattern(pos + 8);
}

function deserializeDebuggerStatement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'DebuggerStatement',
    start,
    end,
    range: [start, end],
  };
}

function deserializeBindingPattern(pos) {
  let pattern = deserializeBindingPatternKind(pos);
  pattern.optional = deserializeBool(pos + 24);
  pattern.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 16);
  return pattern;
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for BindingPatternKind`);
  }
}

function deserializeAssignmentPattern(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'AssignmentPattern',
    decorators: [],
    left: deserializeBindingPattern(pos + 8),
    right: deserializeExpression(pos + 40),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeObjectPattern(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    properties = deserializeVecBindingProperty(pos + 8),
    rest = deserializeOptionBoxBindingRestElement(pos + 32);
  rest !== null && properties.push(rest);
  return {
    type: 'ObjectPattern',
    decorators: [],
    properties,
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeBindingProperty(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Property',
    kind: 'init',
    key: deserializePropertyKey(pos + 8),
    value: deserializeBindingPattern(pos + 24),
    method: false,
    shorthand: deserializeBool(pos + 56),
    computed: deserializeBool(pos + 57),
    optional: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeArrayPattern(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    elements = deserializeVecOptionBindingPattern(pos + 8),
    rest = deserializeOptionBoxBindingRestElement(pos + 32);
  rest !== null && elements.push(rest);
  return {
    type: 'ArrayPattern',
    decorators: [],
    elements,
    optional: false,
    typeAnnotation: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeBindingRestElement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'RestElement',
    decorators: [],
    argument: deserializeBindingPattern(pos + 8),
    optional: false,
    typeAnnotation: null,
    value: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeFunction(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), params = deserializeBoxFormalParameters(pos + 56);
  {
    let thisParam = deserializeOptionBoxTSThisParameter(pos + 48);
    thisParam !== null && params.unshift(thisParam);
  }
  return {
    type: deserializeFunctionType(pos + 84),
    id: deserializeOptionBindingIdentifier(pos + 8),
    generator: deserializeBool(pos + 85),
    async: deserializeBool(pos + 86),
    declare: deserializeBool(pos + 87),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 40),
    params,
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 64),
    body: deserializeOptionBoxFunctionBody(pos + 72),
    expression: false,
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for FunctionType`);
  }
}

function deserializeFormalParameters(pos) {
  let params = deserializeVecFormalParameter(pos + 8);
  if (uint32[pos + 32 >> 2] !== 0 && uint32[pos + 36 >> 2] !== 0) {
    pos = uint32[pos + 32 >> 2];
    let start, end;
    params.push({
      type: 'RestElement',
      decorators: [],
      argument: deserializeBindingPatternKind(pos + 8),
      optional: deserializeBool(pos + 32),
      typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 24),
      value: null,
      start: start = deserializeU32(pos),
      end: end = deserializeU32(pos + 4),
      range: [start, end],
    });
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param;
  {
    let accessibility = deserializeOptionTSAccessibility(pos + 64),
      readonly = deserializeBool(pos + 65),
      override = deserializeBool(pos + 66);
    if (accessibility === null && !readonly && !override) {
      param = deserializeBindingPatternKind(pos + 32);
      param.decorators = deserializeVecDecorator(pos + 8);
      param.optional = deserializeBool(pos + 56);
      param.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 48);
    } else {
      let start, end;
      param = {
        type: 'TSParameterProperty',
        accessibility,
        decorators: deserializeVecDecorator(pos + 8),
        override,
        parameter: deserializeBindingPattern(pos + 32),
        readonly,
        static: false,
        start: start = deserializeU32(pos),
        end: end = deserializeU32(pos + 4),
        range: [start, end],
      };
    }
  }
  return param;
}

function deserializeFunctionBody(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 32));
  return {
    type: 'BlockStatement',
    body,
    start,
    end,
    range: [start, end],
  };
}

function deserializeArrowFunctionExpression(pos) {
  let expression = deserializeBool(pos + 44),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    body = deserializeBoxFunctionBody(pos + 32);
  return {
    type: 'ArrowFunctionExpression',
    expression,
    async: deserializeBool(pos + 45),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    params: deserializeBoxFormalParameters(pos + 16),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 24),
    body: expression ? body.body[0].expression : body,
    id: null,
    generator: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeYieldExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'YieldExpression',
    delegate: deserializeBool(pos + 24),
    argument: deserializeOptionExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeClass(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: deserializeClassType(pos + 132),
    decorators: deserializeVecDecorator(pos + 8),
    id: deserializeOptionBindingIdentifier(pos + 32),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 64),
    superClass: deserializeOptionExpression(pos + 72),
    superTypeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 88),
    implements: deserializeVecTSClassImplements(pos + 96),
    body: deserializeBoxClassBody(pos + 120),
    abstract: deserializeBool(pos + 133),
    declare: deserializeBool(pos + 134),
    start,
    end,
    range: [start, end],
  };
}

function deserializeClassType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'ClassDeclaration';
    case 1:
      return 'ClassExpression';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ClassType`);
  }
}

function deserializeClassBody(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ClassBody',
    body: deserializeVecClassElement(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ClassElement`);
  }
}

function deserializeMethodDefinition(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: deserializeMethodDefinitionType(pos + 56),
    decorators: deserializeVecDecorator(pos + 8),
    key: deserializePropertyKey(pos + 32),
    value: deserializeBoxFunction(pos + 48),
    kind: deserializeMethodDefinitionKind(pos + 57),
    computed: deserializeBool(pos + 58),
    static: deserializeBool(pos + 59),
    override: deserializeBool(pos + 60),
    optional: deserializeBool(pos + 61),
    accessibility: deserializeOptionTSAccessibility(pos + 62),
    start,
    end,
    range: [start, end],
  };
}

function deserializeMethodDefinitionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'MethodDefinition';
    case 1:
      return 'TSAbstractMethodDefinition';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for MethodDefinitionType`);
  }
}

function deserializePropertyDefinition(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: deserializePropertyDefinitionType(pos + 72),
    decorators: deserializeVecDecorator(pos + 8),
    key: deserializePropertyKey(pos + 32),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 48),
    value: deserializeOptionExpression(pos + 56),
    computed: deserializeBool(pos + 73),
    static: deserializeBool(pos + 74),
    declare: deserializeBool(pos + 75),
    override: deserializeBool(pos + 76),
    optional: deserializeBool(pos + 77),
    definite: deserializeBool(pos + 78),
    readonly: deserializeBool(pos + 79),
    accessibility: deserializeOptionTSAccessibility(pos + 80),
    start,
    end,
    range: [start, end],
  };
}

function deserializePropertyDefinitionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'PropertyDefinition';
    case 1:
      return 'TSAbstractPropertyDefinition';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for PropertyDefinitionType`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for MethodDefinitionKind`);
  }
}

function deserializePrivateIdentifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'PrivateIdentifier',
    name: deserializeStr(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeStaticBlock(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'StaticBlock',
    body: deserializeVecStatement(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeAccessorPropertyType(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'AccessorProperty';
    case 1:
      return 'TSAbstractAccessorProperty';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AccessorPropertyType`);
  }
}

function deserializeAccessorProperty(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: deserializeAccessorPropertyType(pos + 72),
    decorators: deserializeVecDecorator(pos + 8),
    key: deserializePropertyKey(pos + 32),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 48),
    value: deserializeOptionExpression(pos + 56),
    computed: deserializeBool(pos + 73),
    static: deserializeBool(pos + 74),
    override: deserializeBool(pos + 75),
    definite: deserializeBool(pos + 76),
    accessibility: deserializeOptionTSAccessibility(pos + 77),
    declare: false,
    optional: false,
    readonly: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ImportExpression',
    source: deserializeExpression(pos + 8),
    options: deserializeOptionExpression(pos + 24),
    phase: deserializeOptionImportPhase(pos + 40),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 8);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 80);
  return {
    type: 'ImportDeclaration',
    specifiers,
    source: deserializeStringLiteral(pos + 32),
    phase: deserializeOptionImportPhase(pos + 88),
    attributes: withClause === null ? [] : withClause.attributes,
    importKind: deserializeImportOrExportKind(pos + 89),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportPhase(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'source';
    case 1:
      return 'defer';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportPhase`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportDeclarationSpecifier`);
  }
}

function deserializeImportSpecifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ImportSpecifier',
    imported: deserializeModuleExportName(pos + 8),
    local: deserializeBindingIdentifier(pos + 64),
    importKind: deserializeImportOrExportKind(pos + 96),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportDefaultSpecifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ImportDefaultSpecifier',
    local: deserializeBindingIdentifier(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportNamespaceSpecifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ImportNamespaceSpecifier',
    local: deserializeBindingIdentifier(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 8) };
}

function deserializeImportAttribute(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ImportAttribute',
    key: deserializeImportAttributeKey(pos + 8),
    value: deserializeStringLiteral(pos + 64),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportAttributeKey(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(pos + 8);
    case 1:
      return deserializeStringLiteral(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportAttributeKey`);
  }
}

function deserializeExportNamedDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), withClause = deserializeOptionBoxWithClause(pos + 96);
  return {
    type: 'ExportNamedDeclaration',
    declaration: deserializeOptionDeclaration(pos + 8),
    specifiers: deserializeVecExportSpecifier(pos + 24),
    source: deserializeOptionStringLiteral(pos + 48),
    exportKind: deserializeImportOrExportKind(pos + 104),
    attributes: withClause === null ? [] : withClause.attributes,
    start,
    end,
    range: [start, end],
  };
}

function deserializeExportDefaultDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ExportDefaultDeclaration',
    declaration: deserializeExportDefaultDeclarationKind(pos + 8),
    exportKind: 'value',
    start,
    end,
    range: [start, end],
  };
}

function deserializeExportAllDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    withClause = deserializeOptionBoxWithClause(pos + 112);
  return {
    type: 'ExportAllDeclaration',
    exported: deserializeOptionModuleExportName(pos + 8),
    source: deserializeStringLiteral(pos + 64),
    attributes: withClause === null ? [] : withClause.attributes,
    exportKind: deserializeImportOrExportKind(pos + 120),
    start,
    end,
    range: [start, end],
  };
}

function deserializeExportSpecifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'ExportSpecifier',
    local: deserializeModuleExportName(pos + 8),
    exported: deserializeModuleExportName(pos + 64),
    exportKind: deserializeImportOrExportKind(pos + 120),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportDefaultDeclarationKind`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ModuleExportName`);
  }
}

function deserializeV8IntrinsicExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'V8IntrinsicExpression',
    name: deserializeIdentifierName(pos + 8),
    arguments: deserializeVecArgument(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeBooleanLiteral(pos) {
  let value = deserializeBool(pos + 8), start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Literal',
    value,
    raw: start === 0 && end === 0 ? null : value + '',
    start,
    end,
    range: [start, end],
  };
}

function deserializeNullLiteral(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Literal',
    value: null,
    raw: start === 0 && end === 0 ? null : 'null',
    start,
    end,
    range: [start, end],
  };
}

function deserializeNumericLiteral(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Literal',
    value: deserializeF64(pos + 8),
    raw: deserializeOptionStr(pos + 16),
    start,
    end,
    range: [start, end],
  };
}

function deserializeStringLiteral(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), value = deserializeStr(pos + 8);
  deserializeBool(pos + 40) &&
    (value = value.replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16))));
  return {
    type: 'Literal',
    value,
    raw: deserializeOptionStr(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeBigIntLiteral(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), bigint = deserializeStr(pos + 8);
  return {
    type: 'Literal',
    value: BigInt(bigint),
    raw: deserializeOptionStr(pos + 24),
    bigint,
    start,
    end,
    range: [start, end],
  };
}

function deserializeRegExpLiteral(pos) {
  let regex = deserializeRegExp(pos + 8), start = deserializeU32(pos), end = deserializeU32(pos + 4), value = null;
  try {
    value = new RegExp(regex.pattern, regex.flags);
  } catch {}
  return {
    type: 'Literal',
    value,
    raw: deserializeOptionStr(pos + 40),
    regex,
    start,
    end,
    range: [start, end],
  };
}

function deserializeRegExp(pos) {
  return {
    pattern: deserializeStr(pos),
    flags: deserializeRegExpFlags(pos + 24),
  };
}

function deserializeRegExpFlags(pos) {
  let flagBits = deserializeU8(pos), flags = '';
  // Alphabetical order
  flagBits & 64 && (flags += 'd');
  flagBits & 1 && (flags += 'g');
  flagBits & 2 && (flags += 'i');
  flagBits & 4 && (flags += 'm');
  flagBits & 8 && (flags += 's');
  flagBits & 16 && (flags += 'u');
  flagBits & 128 && (flags += 'v');
  flagBits & 32 && (flags += 'y');
  return flags;
}

function deserializeJSXElement(pos) {
  let closingElement = deserializeOptionBoxJSXClosingElement(pos + 40),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    openingElement = deserializeBoxJSXOpeningElement(pos + 8);
  closingElement === null && (openingElement.selfClosing = true);
  return {
    type: 'JSXElement',
    openingElement,
    children: deserializeVecJSXChild(pos + 16),
    closingElement,
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXOpeningElement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXOpeningElement',
    name: deserializeJSXElementName(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    attributes: deserializeVecJSXAttributeItem(pos + 32),
    selfClosing: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXClosingElement(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXClosingElement',
    name: deserializeJSXElementName(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXFragment(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXFragment',
    openingFragment: deserializeJSXOpeningFragment(pos + 8),
    children: deserializeVecJSXChild(pos + 16),
    closingFragment: deserializeJSXClosingFragment(pos + 40),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXOpeningFragment(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXOpeningFragment',
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXClosingFragment(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXClosingFragment',
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXElementName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXIdentifier(pos + 8);
    case 1:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return {
        type: 'JSXIdentifier',
        name: ident.name,
        start: ident.start,
        end: ident.end,
        range: ident.range,
      };
    case 2:
      return deserializeBoxJSXNamespacedName(pos + 8);
    case 3:
      return deserializeBoxJSXMemberExpression(pos + 8);
    case 4:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return {
        type: 'JSXIdentifier',
        name: 'this',
        start: thisExpr.start,
        end: thisExpr.end,
        range: thisExpr.range,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXNamespacedName',
    namespace: deserializeJSXIdentifier(pos + 8),
    name: deserializeJSXIdentifier(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXMemberExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXMemberExpression',
    object: deserializeJSXMemberExpressionObject(pos + 8),
    property: deserializeJSXIdentifier(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXMemberExpressionObject(pos) {
  switch (uint8[pos]) {
    case 0:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return {
        type: 'JSXIdentifier',
        name: ident.name,
        start: ident.start,
        end: ident.end,
        range: ident.range,
      };
    case 1:
      return deserializeBoxJSXMemberExpression(pos + 8);
    case 2:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return {
        type: 'JSXIdentifier',
        name: 'this',
        start: thisExpr.start,
        end: thisExpr.end,
        range: thisExpr.range,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXExpressionContainer',
    expression: deserializeJSXExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXExpression`);
  }
}

function deserializeJSXEmptyExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXEmptyExpression',
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXAttributeItem(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXAttribute(pos + 8);
    case 1:
      return deserializeBoxJSXSpreadAttribute(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeItem`);
  }
}

function deserializeJSXAttribute(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXAttribute',
    name: deserializeJSXAttributeName(pos + 8),
    value: deserializeOptionJSXAttributeValue(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXSpreadAttribute(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXSpreadAttribute',
    argument: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXAttributeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXIdentifier(pos + 8);
    case 1:
      return deserializeBoxJSXNamespacedName(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeName`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeValue`);
  }
}

function deserializeJSXIdentifier(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXIdentifier',
    name: deserializeStr(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXChild`);
  }
}

function deserializeJSXSpreadChild(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXSpreadChild',
    expression: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSXText(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'JSXText',
    value: deserializeStr(pos + 8),
    raw: deserializeOptionStr(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSThisParameter(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Identifier',
    decorators: [],
    name: 'this',
    optional: false,
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 16),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSEnumDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSEnumDeclaration',
    id: deserializeBindingIdentifier(pos + 8),
    body: deserializeTSEnumBody(pos + 40),
    const: deserializeBool(pos + 76),
    declare: deserializeBool(pos + 77),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSEnumBody(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSEnumBody',
    members: deserializeVecTSEnumMember(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSEnumMember(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSEnumMember',
    id: deserializeTSEnumMemberName(pos + 8),
    initializer: deserializeOptionExpression(pos + 24),
    computed: deserializeU8(pos + 8) > 1,
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSEnumMemberName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierName(pos + 8);
    case 1:
      return deserializeBoxStringLiteral(pos + 8);
    case 2:
      return deserializeBoxStringLiteral(pos + 8);
    case 3:
      return deserializeBoxTemplateLiteral(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSEnumMemberName`);
  }
}

function deserializeTSTypeAnnotation(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeAnnotation',
    typeAnnotation: deserializeTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSLiteralType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSLiteralType',
    literal: deserializeTSLiteral(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSLiteral`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSType`);
  }
}

function deserializeTSConditionalType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSConditionalType',
    checkType: deserializeTSType(pos + 8),
    extendsType: deserializeTSType(pos + 24),
    trueType: deserializeTSType(pos + 40),
    falseType: deserializeTSType(pos + 56),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSUnionType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSUnionType',
    types: deserializeVecTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSIntersectionType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSIntersectionType',
    types: deserializeVecTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSParenthesizedType(pos) {
  return deserializeTSType(pos + 8);
}

function deserializeTSTypeOperator(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeOperator',
    operator: deserializeTSTypeOperatorOperator(pos + 24),
    typeAnnotation: deserializeTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeOperatorOperator`);
  }
}

function deserializeTSArrayType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSArrayType',
    elementType: deserializeTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSIndexedAccessType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSIndexedAccessType',
    objectType: deserializeTSType(pos + 8),
    indexType: deserializeTSType(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTupleType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTupleType',
    elementTypes: deserializeVecTSTupleElement(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSNamedTupleMember(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSNamedTupleMember',
    label: deserializeIdentifierName(pos + 8),
    elementType: deserializeTSTupleElement(pos + 32),
    optional: deserializeBool(pos + 48),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSOptionalType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSOptionalType',
    typeAnnotation: deserializeTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSRestType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSRestType',
    typeAnnotation: deserializeTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTupleElement`);
  }
}

function deserializeTSAnyKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSAnyKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSStringKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSStringKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSBooleanKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSBooleanKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSNumberKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSNumberKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSNeverKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSNeverKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSIntrinsicKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSUnknownKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSUnknownKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSNullKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSNullKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSUndefinedKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSUndefinedKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSVoidKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSVoidKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSSymbolKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSSymbolKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSThisType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSThisType',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSObjectKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSObjectKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSBigIntKeyword(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSBigIntKeyword',
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeReference(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeReference',
    typeName: deserializeTSTypeName(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSQualifiedName(pos + 8);
    case 2:
      return deserializeBoxThisExpression(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeName`);
  }
}

function deserializeTSQualifiedName(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSQualifiedName',
    left: deserializeTSTypeName(pos + 8),
    right: deserializeIdentifierName(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeParameterInstantiation(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeParameterInstantiation',
    params: deserializeVecTSType(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeParameter(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeParameter',
    name: deserializeBindingIdentifier(pos + 8),
    constraint: deserializeOptionTSType(pos + 40),
    default: deserializeOptionTSType(pos + 56),
    in: deserializeBool(pos + 72),
    out: deserializeBool(pos + 73),
    const: deserializeBool(pos + 74),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeParameterDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeParameterDeclaration',
    params: deserializeVecTSTypeParameter(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeAliasDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeAliasDeclaration',
    id: deserializeBindingIdentifier(pos + 8),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 40),
    typeAnnotation: deserializeTSType(pos + 48),
    declare: deserializeBool(pos + 68),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSAccessibility`);
  }
}

function deserializeTSClassImplements(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), expression = deserializeTSTypeName(pos + 8);
  if (expression.type === 'TSQualifiedName') {
    let object = expression.left,
      start,
      end,
      parent = expression = {
        type: 'MemberExpression',
        object,
        property: expression.right,
        optional: false,
        computed: false,
        start: start = expression.start,
        end: end = expression.end,
        range: [start, end],
      };
    for (; object.type === 'TSQualifiedName';) {
      let { left } = object, start, end;
      parent = parent.object = {
        type: 'MemberExpression',
        object: left,
        property: object.right,
        optional: false,
        computed: false,
        start: start = object.start,
        end: end = object.end,
        range: [start, end],
      };
      object = left;
    }
  }
  return {
    type: 'TSClassImplements',
    expression,
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSInterfaceDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSInterfaceDeclaration',
    id: deserializeBindingIdentifier(pos + 8),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 40),
    extends: deserializeVecTSInterfaceHeritage(pos + 48),
    body: deserializeBoxTSInterfaceBody(pos + 72),
    declare: deserializeBool(pos + 84),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSInterfaceBody(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSInterfaceBody',
    body: deserializeVecTSSignature(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSPropertySignature(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSPropertySignature',
    computed: deserializeBool(pos + 32),
    optional: deserializeBool(pos + 33),
    readonly: deserializeBool(pos + 34),
    key: deserializePropertyKey(pos + 8),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 24),
    accessibility: null,
    static: false,
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSSignature`);
  }
}

function deserializeTSIndexSignature(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSIndexSignature',
    parameters: deserializeVecTSIndexSignatureName(pos + 8),
    typeAnnotation: deserializeBoxTSTypeAnnotation(pos + 32),
    readonly: deserializeBool(pos + 40),
    static: deserializeBool(pos + 41),
    accessibility: null,
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSCallSignatureDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    params = deserializeBoxFormalParameters(pos + 24),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 16);
  thisParam !== null && params.unshift(thisParam);
  return {
    type: 'TSCallSignatureDeclaration',
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    params,
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 32),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSMethodSignatureKind`);
  }
}

function deserializeTSMethodSignature(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    params = deserializeBoxFormalParameters(pos + 40),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 32);
  thisParam !== null && params.unshift(thisParam);
  return {
    type: 'TSMethodSignature',
    key: deserializePropertyKey(pos + 8),
    computed: deserializeBool(pos + 60),
    optional: deserializeBool(pos + 61),
    kind: deserializeTSMethodSignatureKind(pos + 62),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 24),
    params,
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 48),
    accessibility: null,
    readonly: false,
    static: false,
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSConstructSignatureDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSConstructSignatureDeclaration',
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    params: deserializeBoxFormalParameters(pos + 16),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSIndexSignatureName(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Identifier',
    decorators: [],
    name: deserializeStr(pos + 8),
    optional: false,
    typeAnnotation: deserializeBoxTSTypeAnnotation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSInterfaceHeritage(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSInterfaceHeritage',
    expression: deserializeExpression(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypePredicate(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypePredicate',
    parameterName: deserializeTSTypePredicateName(pos + 8),
    asserts: deserializeBool(pos + 32),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypePredicateName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierName(pos + 8);
    case 1:
      return deserializeTSThisType(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypePredicateName`);
  }
}

function deserializeTSModuleDeclaration(pos) {
  let kind = deserializeTSModuleDeclarationKind(pos + 84),
    global = kind === 'global',
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    declare = deserializeBool(pos + 85),
    id = deserializeTSModuleDeclarationName(pos + 8),
    body = deserializeOptionTSModuleDeclarationBody(pos + 64);
  // Flatten `body`, and nest `id`
  if (body !== null && body.type === 'TSModuleDeclaration') {
    let innerId = body.id;
    if (innerId.type === 'Identifier') {
      let start, end;
      id = {
        type: 'TSQualifiedName',
        left: id,
        right: innerId,
        start: start = id.start,
        end: end = innerId.end,
        range: [start, end],
      };
    } else {
      // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with `id` of
      // this module on left, and previous `left` of innermost `TSQualifiedName` on right
      for (;;) {
        innerId.start = innerId.range[0] = id.start;
        if (innerId.left.type === 'Identifier') break;
        innerId = innerId.left;
      }
      let start, end;
      innerId.left = {
        type: 'TSQualifiedName',
        left: id,
        right: innerId.left,
        start: start = id.start,
        end: end = innerId.left.end,
        range: [start, end],
      };
      id = body.id;
    }
    body = Object.hasOwn(body, 'body') ? body.body : null;
  }
  return body === null
    ? {
      type: 'TSModuleDeclaration',
      id,
      kind,
      declare,
      global,
      start,
      end,
      range: [start, end],
    }
    : {
      type: 'TSModuleDeclaration',
      id,
      body,
      kind,
      declare,
      global,
      start,
      end,
      range: [start, end],
    };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleDeclarationKind`);
  }
}

function deserializeTSModuleDeclarationName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBindingIdentifier(pos + 8);
    case 1:
      return deserializeStringLiteral(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleDeclarationName`);
  }
}

function deserializeTSModuleDeclarationBody(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxTSModuleDeclaration(pos + 8);
    case 1:
      return deserializeBoxTSModuleBlock(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleDeclarationBody`);
  }
}

function deserializeTSModuleBlock(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4), body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 32));
  return {
    type: 'TSModuleBlock',
    body,
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeLiteral(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeLiteral',
    members: deserializeVecTSSignature(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSInferType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSInferType',
    typeParameter: deserializeBoxTSTypeParameter(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeQuery(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeQuery',
    exprName: deserializeTSTypeQueryExprName(pos + 8),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeQueryExprName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSQualifiedName(pos + 8);
    case 2:
      return deserializeBoxThisExpression(pos + 8);
    case 3:
      return deserializeBoxTSImportType(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeQueryExprName`);
  }
}

function deserializeTSImportType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSImportType',
    argument: deserializeTSType(pos + 8),
    options: deserializeOptionBoxObjectExpression(pos + 24),
    qualifier: deserializeOptionTSImportTypeQualifier(pos + 32),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 48),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSImportTypeQualifier(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierName(pos + 8);
    case 1:
      return deserializeBoxTSImportTypeQualifiedName(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSImportTypeQualifier`);
  }
}

function deserializeTSImportTypeQualifiedName(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSQualifiedName',
    left: deserializeTSImportTypeQualifier(pos + 8),
    right: deserializeIdentifierName(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSFunctionType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    params = deserializeBoxFormalParameters(pos + 24),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 16);
  thisParam !== null && params.unshift(thisParam);
  return {
    type: 'TSFunctionType',
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    params,
    returnType: deserializeBoxTSTypeAnnotation(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSConstructorType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSConstructorType',
    abstract: deserializeBool(pos + 32),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 8),
    params: deserializeBoxFormalParameters(pos + 16),
    returnType: deserializeBoxTSTypeAnnotation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSMappedType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    typeParameter = deserializeBoxTSTypeParameter(pos + 8),
    optional = deserializeOptionTSMappedTypeModifierOperator(pos + 52);
  optional === null && (optional = false);
  return {
    type: 'TSMappedType',
    key: typeParameter.name,
    constraint: typeParameter.constraint,
    nameType: deserializeOptionTSType(pos + 16),
    typeAnnotation: deserializeOptionTSType(pos + 32),
    optional,
    readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 53),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSMappedTypeModifierOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return true;
    case 1:
      return '+';
    case 2:
      return '-';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSMappedTypeModifierOperator`);
  }
}

function deserializeTSTemplateLiteralType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTemplateLiteralType',
    quasis: deserializeVecTemplateElement(pos + 8),
    types: deserializeVecTSType(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSAsExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSAsExpression',
    expression: deserializeExpression(pos + 8),
    typeAnnotation: deserializeTSType(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSSatisfiesExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSSatisfiesExpression',
    expression: deserializeExpression(pos + 8),
    typeAnnotation: deserializeTSType(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSTypeAssertion(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSTypeAssertion',
    typeAnnotation: deserializeTSType(pos + 8),
    expression: deserializeExpression(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSImportEqualsDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSImportEqualsDeclaration',
    id: deserializeBindingIdentifier(pos + 8),
    moduleReference: deserializeTSModuleReference(pos + 40),
    importKind: deserializeImportOrExportKind(pos + 56),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSModuleReference(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierReference(pos + 8);
    case 1:
      return deserializeBoxTSQualifiedName(pos + 8);
    case 2:
      return deserializeBoxThisExpression(pos + 8);
    case 3:
      return deserializeBoxTSExternalModuleReference(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleReference`);
  }
}

function deserializeTSExternalModuleReference(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSExternalModuleReference',
    expression: deserializeStringLiteral(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSNonNullExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSNonNullExpression',
    expression: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeDecorator(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'Decorator',
    expression: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSExportAssignment(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSExportAssignment',
    expression: deserializeExpression(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSNamespaceExportDeclaration',
    id: deserializeIdentifierName(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeTSInstantiationExpression(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSInstantiationExpression',
    expression: deserializeExpression(pos + 8),
    typeArguments: deserializeBoxTSTypeParameterInstantiation(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportOrExportKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'value';
    case 1:
      return 'type';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportOrExportKind`);
  }
}

function deserializeJSDocNullableType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSJSDocNullableType',
    typeAnnotation: deserializeTSType(pos + 8),
    postfix: deserializeBool(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSDocNonNullableType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSJSDocNonNullableType',
    typeAnnotation: deserializeTSType(pos + 8),
    postfix: deserializeBool(pos + 24),
    start,
    end,
    range: [start, end],
  };
}

function deserializeJSDocUnknownType(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    type: 'TSJSDocUnknownType',
    start,
    end,
    range: [start, end],
  };
}

function deserializeCommentKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'Line';
    case 1:
      return 'Block';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for CommentKind`);
  }
}

function deserializeComment(pos) {
  let type = deserializeCommentKind(pos + 12),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    endCut = type === 'Line' ? 0 : 2;
  return {
    type,
    value: sourceText.slice(start + 2, end - endCut),
    start,
    end,
    range: [start, end],
  };
}

function deserializeNameSpan(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    value: deserializeStr(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeImportEntry(pos) {
  return {
    importName: deserializeImportImportName(pos + 32),
    localName: deserializeNameSpan(pos + 64),
    isType: deserializeBool(pos + 88),
  };
}

function deserializeImportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: 'Name',
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      return {
        kind: 'NamespaceObject',
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    case 2:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: 'Default',
        name: null,
        start,
        end,
        range: [start, end],
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportImportName`);
  }
}

function deserializeExportEntry(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeOptionNameSpan(pos + 16),
    importName: deserializeExportImportName(pos + 40),
    exportName: deserializeExportExportName(pos + 72),
    localName: deserializeExportLocalName(pos + 104),
    isType: deserializeBool(pos + 136),
    start,
    end,
    range: [start, end],
  };
}

function deserializeExportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: 'Name',
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      return {
        kind: 'All',
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    case 2:
      return {
        kind: 'AllButDefault',
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    case 3:
      return {
        kind: 'None',
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportImportName`);
  }
}

function deserializeExportExportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: 'Name',
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: 'Default',
        name: null,
        start,
        end,
        range: [start, end],
      };
    case 2:
      return {
        kind: 'None',
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportExportName`);
  }
}

function deserializeExportLocalName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: 'Name',
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: 'Default',
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 2:
      return {
        kind: 'None',
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportLocalName`);
  }
}

function deserializeDynamicImport(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeSpan(pos + 8),
    start,
    end,
    range: [start, end],
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentOperator`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for BinaryOperator`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for LogicalOperator`);
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for UnaryOperator`);
  }
}

function deserializeUpdateOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return '++';
    case 1:
      return '--';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for UpdateOperator`);
  }
}

function deserializeSpan(pos) {
  return {
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeModuleKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return 'script';
    case 1:
      return 'module';
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ModuleKind`);
  }
}

function deserializeRawTransferData(pos) {
  return {
    program: deserializeProgram(pos),
    comments: deserializeVecComment(pos + 128),
    module: deserializeEcmaScriptModule(pos + 152),
    errors: deserializeVecError(pos + 256),
  };
}

function deserializeError(pos) {
  return {
    severity: deserializeErrorSeverity(pos + 72),
    message: deserializeStr(pos),
    labels: deserializeVecErrorLabel(pos + 16),
    helpMessage: deserializeOptionStr(pos + 40),
    codeframe: deserializeStr(pos + 56),
  };
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for ErrorSeverity`);
  }
}

function deserializeErrorLabel(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    message: deserializeOptionStr(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeEcmaScriptModule(pos) {
  return {
    hasModuleSyntax: deserializeBool(pos + 96),
    staticImports: deserializeVecStaticImport(pos),
    staticExports: deserializeVecStaticExport(pos + 24),
    dynamicImports: deserializeVecDynamicImport(pos + 48),
    importMetas: deserializeVecSpan(pos + 72),
  };
}

function deserializeStaticImport(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeNameSpan(pos + 8),
    entries: deserializeVecImportEntry(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeStaticExport(pos) {
  let start = deserializeU32(pos), end = deserializeU32(pos + 4);
  return {
    entries: deserializeVecExportEntry(pos + 8),
    start,
    end,
    range: [start, end],
  };
}

function deserializeU32(pos) {
  return uint32[pos >> 2];
}

function deserializeU8(pos) {
  return uint8[pos];
}

function deserializeStr(pos) {
  let pos32 = pos >> 2, len = uint32[pos32 + 2];
  if (len === 0) return '';
  pos = uint32[pos32];
  if (sourceIsAscii && pos < sourceByteLen) return sourceText.substr(pos, len);
  // Longer strings use `TextDecoder`
  // TODO: Find best switch-over point
  let end = pos + len;
  if (len > 50) return decodeStr(uint8.subarray(pos, end));
  // Shorter strings decode by hand to avoid native call
  let out = '', c;
  do {
    c = uint8[pos++];
    if (c < 128) out += fromCodePoint(c);
    else {
      out += decodeStr(uint8.subarray(pos - 1, end));
      break;
    }
  } while (pos < end);
  return out;
}

function deserializeVecComment(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeComment(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionHashbang(pos) {
  if (uint32[pos + 8 >> 2] === 0 && uint32[pos + 12 >> 2] === 0) return null;
  return deserializeHashbang(pos);
}

function deserializeVecDirective(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 72;
  for (; pos !== endPos;) {
    arr.push(deserializeDirective(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecStatement(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeStatement(pos));
    pos += 16;
  }
  return arr;
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

function deserializeVecArrayExpressionElement(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeArrayExpressionElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxSpreadElement(pos) {
  return deserializeSpreadElement(uint32[pos >> 2]);
}

function deserializeVecObjectPropertyKind(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 48;
  for (; pos !== endPos;) {
    arr.push(deserializeTemplateElement(pos));
    pos += 48;
  }
  return arr;
}

function deserializeVecExpression(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeExpression(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSTypeParameterInstantiation(pos) {
  return deserializeTSTypeParameterInstantiation(uint32[pos >> 2]);
}

function deserializeOptionBoxTSTypeParameterInstantiation(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxTSTypeParameterInstantiation(pos);
}

function deserializeOptionStr(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeOptionAssignmentTargetMaybeDefault(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxAssignmentTargetRest(pos) {
  return deserializeAssignmentTargetRest(uint32[pos >> 2]);
}

function deserializeOptionBoxAssignmentTargetRest(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxAssignmentTargetRest(pos);
}

function deserializeVecAssignmentTargetProperty(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 64;
  for (; pos !== endPos;) {
    arr.push(deserializeVariableDeclarator(pos));
    pos += 64;
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
  if (uint32[pos + 8 >> 2] === 0 && uint32[pos + 12 >> 2] === 0) return null;
  return deserializeLabelIdentifier(pos);
}

function deserializeVecSwitchCase(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 48;
  for (; pos !== endPos;) {
    arr.push(deserializeSwitchCase(pos));
    pos += 48;
  }
  return arr;
}

function deserializeBoxCatchClause(pos) {
  return deserializeCatchClause(uint32[pos >> 2]);
}

function deserializeOptionBoxCatchClause(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxCatchClause(pos);
}

function deserializeOptionBoxBlockStatement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
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
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 64;
  for (; pos !== endPos;) {
    arr.push(deserializeBindingProperty(pos));
    pos += 64;
  }
  return arr;
}

function deserializeBoxBindingRestElement(pos) {
  return deserializeBindingRestElement(uint32[pos >> 2]);
}

function deserializeOptionBoxBindingRestElement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxBindingRestElement(pos);
}

function deserializeOptionBindingPattern(pos) {
  if (uint8[pos + 24] === 2) return null;
  return deserializeBindingPattern(pos);
}

function deserializeVecOptionBindingPattern(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos;) {
    arr.push(deserializeOptionBindingPattern(pos));
    pos += 32;
  }
  return arr;
}

function deserializeOptionBindingIdentifier(pos) {
  if (uint32[pos + 8 >> 2] === 0 && uint32[pos + 12 >> 2] === 0) return null;
  return deserializeBindingIdentifier(pos);
}

function deserializeBoxTSTypeParameterDeclaration(pos) {
  return deserializeTSTypeParameterDeclaration(uint32[pos >> 2]);
}

function deserializeOptionBoxTSTypeParameterDeclaration(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxTSTypeParameterDeclaration(pos);
}

function deserializeBoxTSThisParameter(pos) {
  return deserializeTSThisParameter(uint32[pos >> 2]);
}

function deserializeOptionBoxTSThisParameter(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxTSThisParameter(pos);
}

function deserializeBoxFormalParameters(pos) {
  return deserializeFormalParameters(uint32[pos >> 2]);
}

function deserializeBoxFunctionBody(pos) {
  return deserializeFunctionBody(uint32[pos >> 2]);
}

function deserializeOptionBoxFunctionBody(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxFunctionBody(pos);
}

function deserializeVecFormalParameter(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 72;
  for (; pos !== endPos;) {
    arr.push(deserializeFormalParameter(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecDecorator(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 24;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos;) {
    arr.push(deserializeTSClassImplements(pos));
    pos += 32;
  }
  return arr;
}

function deserializeBoxClassBody(pos) {
  return deserializeClassBody(uint32[pos >> 2]);
}

function deserializeVecClassElement(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeImportDeclarationSpecifier(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionVecImportDeclarationSpecifier(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeVecImportDeclarationSpecifier(pos);
}

function deserializeBoxWithClause(pos) {
  return deserializeWithClause(uint32[pos >> 2]);
}

function deserializeOptionBoxWithClause(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 112;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 128;
  for (; pos !== endPos;) {
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

function deserializeBoxJSXOpeningElement(pos) {
  return deserializeJSXOpeningElement(uint32[pos >> 2]);
}

function deserializeVecJSXChild(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeJSXChild(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxJSXClosingElement(pos) {
  return deserializeJSXClosingElement(uint32[pos >> 2]);
}

function deserializeOptionBoxJSXClosingElement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxJSXClosingElement(pos);
}

function deserializeVecJSXAttributeItem(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 40;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeTSType(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecTSTupleElement(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 80;
  for (; pos !== endPos;) {
    arr.push(deserializeTSTypeParameter(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecTSInterfaceHeritage(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos;) {
    arr.push(deserializeTSInterfaceHeritage(pos));
    pos += 32;
  }
  return arr;
}

function deserializeBoxTSInterfaceBody(pos) {
  return deserializeTSInterfaceBody(uint32[pos >> 2]);
}

function deserializeVecTSSignature(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
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
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos;) {
    arr.push(deserializeTSIndexSignatureName(pos));
    pos += 32;
  }
  return arr;
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
  if (uint32[pos >> 2] === 0 && uint32[pos + 4 >> 2] === 0) return null;
  return deserializeBoxObjectExpression(pos);
}

function deserializeOptionTSImportTypeQualifier(pos) {
  if (uint8[pos] === 2) return null;
  return deserializeTSImportTypeQualifier(pos);
}

function deserializeBoxTSImportTypeQualifiedName(pos) {
  return deserializeTSImportTypeQualifiedName(uint32[pos >> 2]);
}

function deserializeOptionTSMappedTypeModifierOperator(pos) {
  if (uint8[pos] === 3) return null;
  return deserializeTSMappedTypeModifierOperator(pos);
}

function deserializeBoxTSExternalModuleReference(pos) {
  return deserializeTSExternalModuleReference(uint32[pos >> 2]);
}

function deserializeOptionNameSpan(pos) {
  if (uint32[pos + 8 >> 2] === 0 && uint32[pos + 12 >> 2] === 0) return null;
  return deserializeNameSpan(pos);
}

function deserializeVecError(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 80;
  for (; pos !== endPos;) {
    arr.push(deserializeError(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecErrorLabel(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 24;
  for (; pos !== endPos;) {
    arr.push(deserializeErrorLabel(pos));
    pos += 24;
  }
  return arr;
}

function deserializeVecStaticImport(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 56;
  for (; pos !== endPos;) {
    arr.push(deserializeStaticImport(pos));
    pos += 56;
  }
  return arr;
}

function deserializeVecStaticExport(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos;) {
    arr.push(deserializeStaticExport(pos));
    pos += 32;
  }
  return arr;
}

function deserializeVecDynamicImport(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos;) {
    arr.push(deserializeDynamicImport(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecSpan(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 8;
  for (; pos !== endPos;) {
    arr.push(deserializeSpan(pos));
    pos += 8;
  }
  return arr;
}

function deserializeVecImportEntry(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 96;
  for (; pos !== endPos;) {
    arr.push(deserializeImportEntry(pos));
    pos += 96;
  }
  return arr;
}

function deserializeVecExportEntry(pos) {
  let arr = [], pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 144;
  for (; pos !== endPos;) {
    arr.push(deserializeExportEntry(pos));
    pos += 144;
  }
  return arr;
}
