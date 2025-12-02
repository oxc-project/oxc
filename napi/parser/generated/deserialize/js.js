// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8, uint32, float64, sourceText, sourceIsAscii, sourceByteLen;

const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true }),
  decodeStr = textDecoder.decode.bind(textDecoder),
  { fromCodePoint } = String;

export function deserialize(buffer, sourceText, sourceByteLen) {
  let data = deserializeWith(buffer, sourceText, sourceByteLen, null, deserializeRawTransferData);
  resetBuffer();
  return data;
}

function deserializeWith(buffer, sourceTextInput, sourceByteLenInput, getLocInput, deserialize) {
  uint8 = buffer;
  uint32 = buffer.uint32;
  float64 = buffer.float64;
  sourceText = sourceTextInput;
  sourceByteLen = sourceByteLenInput;
  sourceIsAscii = sourceText.length === sourceByteLen;
  return deserialize(uint32[536870902]);
}

export function resetBuffer() {
  // Clear buffer and source text string to allow them to be garbage collected
  uint8 = uint32 = float64 = sourceText = void 0;
}

function deserializeProgram(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    program = {
      type: "Program",
      body: null,
      sourceType: deserializeModuleKind(pos + 125),
      hashbang: null,
      start,
      end,
    };
  program.hashbang = deserializeOptionHashbang(pos + 48);
  (program.body = deserializeVecDirective(pos + 72)).push(...deserializeVecStatement(pos + 96));
  return program;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Identifier",
    name: deserializeStr(pos + 8),
    start,
    end,
  };
}

function deserializeIdentifierReference(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Identifier",
    name: deserializeStr(pos + 8),
    start,
    end,
  };
}

function deserializeBindingIdentifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Identifier",
    name: deserializeStr(pos + 8),
    start,
    end,
  };
}

function deserializeLabelIdentifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Identifier",
    name: deserializeStr(pos + 8),
    start,
    end,
  };
}

function deserializeThisExpression(pos) {
  return {
    type: "ThisExpression",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeArrayExpression(pos) {
  let node = {
    type: "ArrayExpression",
    elements: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.elements = deserializeVecArrayExpressionElement(pos + 8);
  return node;
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
  let node = {
    type: "ObjectExpression",
    properties: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.properties = deserializeVecObjectPropertyKind(pos + 8);
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Property",
      kind: deserializePropertyKind(pos + 40),
      key: null,
      value: null,
      method: deserializeBool(pos + 41),
      shorthand: deserializeBool(pos + 42),
      computed: deserializeBool(pos + 43),
      start,
      end,
    };
  node.key = deserializePropertyKey(pos + 8);
  node.value = deserializeExpression(pos + 24);
  return node;
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
      return "init";
    case 1:
      return "get";
    case 2:
      return "set";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for PropertyKind`);
  }
}

function deserializeTemplateLiteral(pos) {
  let node = {
    type: "TemplateLiteral",
    quasis: null,
    expressions: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.quasis = deserializeVecTemplateElement(pos + 8);
  node.expressions = deserializeVecExpression(pos + 32);
  return node;
}

function deserializeTaggedTemplateExpression(pos) {
  let node = {
    type: "TaggedTemplateExpression",
    tag: null,
    quasi: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.tag = deserializeExpression(pos + 8);
  node.quasi = deserializeTemplateLiteral(pos + 32);
  return node;
}

function deserializeTemplateElement(pos) {
  let tail = deserializeBool(pos + 40),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    value = deserializeTemplateElementValue(pos + 8);
  value.cooked !== null &&
    deserializeBool(pos + 41) &&
    (value.cooked = value.cooked.replace(/\uFFFD(.{4})/g, (_, hex) =>
      String.fromCodePoint(parseInt(hex, 16)),
    ));
  return {
    type: "TemplateElement",
    value,
    tail,
    start,
    end,
  };
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 40),
      computed: null,
      start,
      end,
    };
  node.object = deserializeExpression(pos + 8);
  node.property = deserializeExpression(pos + 24);
  node.computed = true;
  return node;
}

function deserializeStaticMemberExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 48),
      computed: null,
      start,
      end,
    };
  node.object = deserializeExpression(pos + 8);
  node.property = deserializeIdentifierName(pos + 24);
  node.computed = false;
  return node;
}

function deserializePrivateFieldExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 48),
      computed: null,
      start,
      end,
    };
  node.object = deserializeExpression(pos + 8);
  node.property = deserializePrivateIdentifier(pos + 24);
  node.computed = false;
  return node;
}

function deserializeCallExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "CallExpression",
      callee: null,
      arguments: null,
      optional: deserializeBool(pos + 56),
      start,
      end,
    };
  node.callee = deserializeExpression(pos + 8);
  node.arguments = deserializeVecArgument(pos + 32);
  return node;
}

function deserializeNewExpression(pos) {
  let node = {
    type: "NewExpression",
    callee: null,
    arguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.callee = deserializeExpression(pos + 8);
  node.arguments = deserializeVecArgument(pos + 32);
  return node;
}

function deserializeMetaProperty(pos) {
  let node = {
    type: "MetaProperty",
    meta: null,
    property: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.meta = deserializeIdentifierName(pos + 8);
  node.property = deserializeIdentifierName(pos + 32);
  return node;
}

function deserializeSpreadElement(pos) {
  let node = {
    type: "SpreadElement",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeExpression(pos + 8);
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "UpdateExpression",
      operator: deserializeUpdateOperator(pos + 24),
      prefix: deserializeBool(pos + 25),
      argument: null,
      start,
      end,
    };
  node.argument = deserializeSimpleAssignmentTarget(pos + 8);
  return node;
}

function deserializeUnaryExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "UnaryExpression",
      operator: deserializeUnaryOperator(pos + 24),
      argument: null,
      prefix: null,
      start,
      end,
    };
  node.argument = deserializeExpression(pos + 8);
  node.prefix = true;
  return node;
}

function deserializeBinaryExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "BinaryExpression",
      left: null,
      operator: deserializeBinaryOperator(pos + 40),
      right: null,
      start,
      end,
    };
  node.left = deserializeExpression(pos + 8);
  node.right = deserializeExpression(pos + 24);
  return node;
}

function deserializePrivateInExpression(pos) {
  let node = {
    type: "BinaryExpression",
    left: null,
    operator: null,
    right: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.left = deserializePrivateIdentifier(pos + 8);
  node.operator = "in";
  node.right = deserializeExpression(pos + 32);
  return node;
}

function deserializeLogicalExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "LogicalExpression",
      left: null,
      operator: deserializeLogicalOperator(pos + 40),
      right: null,
      start,
      end,
    };
  node.left = deserializeExpression(pos + 8);
  node.right = deserializeExpression(pos + 24);
  return node;
}

function deserializeConditionalExpression(pos) {
  let node = {
    type: "ConditionalExpression",
    test: null,
    consequent: null,
    alternate: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.test = deserializeExpression(pos + 8);
  node.consequent = deserializeExpression(pos + 24);
  node.alternate = deserializeExpression(pos + 40);
  return node;
}

function deserializeAssignmentExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "AssignmentExpression",
      operator: deserializeAssignmentOperator(pos + 40),
      left: null,
      right: null,
      start,
      end,
    };
  node.left = deserializeAssignmentTarget(pos + 8);
  node.right = deserializeExpression(pos + 24);
  return node;
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
  let node = {
      type: "ArrayPattern",
      elements: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 8),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 32);
  rest !== null && elements.push(rest);
  node.elements = elements;
  return node;
}

function deserializeObjectAssignmentTarget(pos) {
  let node = {
      type: "ObjectPattern",
      properties: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    properties = deserializeVecAssignmentTargetProperty(pos + 8),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 32);
  rest !== null && properties.push(rest);
  node.properties = properties;
  return node;
}

function deserializeAssignmentTargetRest(pos) {
  let node = {
    type: "RestElement",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeAssignmentTarget(pos + 8);
  return node;
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
  let node = {
    type: "AssignmentPattern",
    left: null,
    right: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.left = deserializeAssignmentTarget(pos + 8);
  node.right = deserializeExpression(pos + 24);
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Property",
      kind: null,
      key: null,
      value: null,
      method: null,
      shorthand: null,
      computed: null,
      start,
      end,
    },
    key = deserializeIdentifierReference(pos + 8),
    value = {
      type: "Identifier",
      name: key.name,
      start: key.start,
      end: key.end,
    },
    init = deserializeOptionExpression(pos + 40);
  init !== null &&
    (value = {
      type: "AssignmentPattern",
      left: value,
      right: init,
      start,
      end,
    });
  node.kind = "init";
  node.key = key;
  node.value = value;
  node.method = false;
  node.shorthand = true;
  node.computed = false;
  return node;
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Property",
      kind: null,
      key: null,
      value: null,
      method: null,
      shorthand: null,
      computed: deserializeBool(pos + 40),
      start,
      end,
    };
  node.kind = "init";
  node.key = deserializePropertyKey(pos + 8);
  node.value = deserializeAssignmentTargetMaybeDefault(pos + 24);
  node.method = false;
  node.shorthand = false;
  return node;
}

function deserializeSequenceExpression(pos) {
  let node = {
    type: "SequenceExpression",
    expressions: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expressions = deserializeVecExpression(pos + 8);
  return node;
}

function deserializeSuper(pos) {
  return {
    type: "Super",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeAwaitExpression(pos) {
  let node = {
    type: "AwaitExpression",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeExpression(pos + 8);
  return node;
}

function deserializeChainExpression(pos) {
  let node = {
    type: "ChainExpression",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeChainElement(pos + 8);
  return node;
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
  let node;
  node = {
    type: "ParenthesizedExpression",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  return node;
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
      return deserializeBoxTSGlobalDeclaration(pos + 8);
    case 40:
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "ExpressionStatement",
      expression: null,
      directive: deserializeStr(pos + 56),
      start,
      end,
    };
  node.expression = deserializeStringLiteral(pos + 8);
  return node;
}

function deserializeHashbang(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Hashbang",
    value: deserializeStr(pos + 8),
    start,
    end,
  };
}

function deserializeBlockStatement(pos) {
  let node = {
    type: "BlockStatement",
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.body = deserializeVecStatement(pos + 8);
  return node;
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
      return deserializeBoxTSGlobalDeclaration(pos + 8);
    case 40:
      return deserializeBoxTSImportEqualsDeclaration(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Declaration`);
  }
}

function deserializeVariableDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "VariableDeclaration",
      kind: deserializeVariableDeclarationKind(pos + 32),
      declarations: null,
      start,
      end,
    };
  node.declarations = deserializeVecVariableDeclarator(pos + 8);
  return node;
}

function deserializeVariableDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "var";
    case 1:
      return "let";
    case 2:
      return "const";
    case 3:
      return "using";
    case 4:
      return "await using";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for VariableDeclarationKind`);
  }
}

function deserializeVariableDeclarator(pos) {
  let node = {
    type: "VariableDeclarator",
    id: null,
    init: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.id = deserializeBindingPattern(pos + 8);
  node.init = deserializeOptionExpression(pos + 40);
  return node;
}

function deserializeEmptyStatement(pos) {
  return {
    type: "EmptyStatement",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeExpressionStatement(pos) {
  let node = {
    type: "ExpressionStatement",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  return node;
}

function deserializeIfStatement(pos) {
  let node = {
    type: "IfStatement",
    test: null,
    consequent: null,
    alternate: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.test = deserializeExpression(pos + 8);
  node.consequent = deserializeStatement(pos + 24);
  node.alternate = deserializeOptionStatement(pos + 40);
  return node;
}

function deserializeDoWhileStatement(pos) {
  let node = {
    type: "DoWhileStatement",
    body: null,
    test: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.body = deserializeStatement(pos + 8);
  node.test = deserializeExpression(pos + 24);
  return node;
}

function deserializeWhileStatement(pos) {
  let node = {
    type: "WhileStatement",
    test: null,
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.test = deserializeExpression(pos + 8);
  node.body = deserializeStatement(pos + 24);
  return node;
}

function deserializeForStatement(pos) {
  let node = {
    type: "ForStatement",
    init: null,
    test: null,
    update: null,
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.init = deserializeOptionForStatementInit(pos + 8);
  node.test = deserializeOptionExpression(pos + 24);
  node.update = deserializeOptionExpression(pos + 40);
  node.body = deserializeStatement(pos + 56);
  return node;
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
  let node = {
    type: "ForInStatement",
    left: null,
    right: null,
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.left = deserializeForStatementLeft(pos + 8);
  node.right = deserializeExpression(pos + 24);
  node.body = deserializeStatement(pos + 40);
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "ForOfStatement",
      await: deserializeBool(pos + 60),
      left: null,
      right: null,
      body: null,
      start,
      end,
    };
  node.left = deserializeForStatementLeft(pos + 8);
  node.right = deserializeExpression(pos + 24);
  node.body = deserializeStatement(pos + 40);
  return node;
}

function deserializeContinueStatement(pos) {
  let node = {
    type: "ContinueStatement",
    label: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.label = deserializeOptionLabelIdentifier(pos + 8);
  return node;
}

function deserializeBreakStatement(pos) {
  let node = {
    type: "BreakStatement",
    label: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.label = deserializeOptionLabelIdentifier(pos + 8);
  return node;
}

function deserializeReturnStatement(pos) {
  let node = {
    type: "ReturnStatement",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeOptionExpression(pos + 8);
  return node;
}

function deserializeWithStatement(pos) {
  let node = {
    type: "WithStatement",
    object: null,
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.object = deserializeExpression(pos + 8);
  node.body = deserializeStatement(pos + 24);
  return node;
}

function deserializeSwitchStatement(pos) {
  let node = {
    type: "SwitchStatement",
    discriminant: null,
    cases: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.discriminant = deserializeExpression(pos + 8);
  node.cases = deserializeVecSwitchCase(pos + 24);
  return node;
}

function deserializeSwitchCase(pos) {
  let node = {
    type: "SwitchCase",
    test: null,
    consequent: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.test = deserializeOptionExpression(pos + 8);
  node.consequent = deserializeVecStatement(pos + 24);
  return node;
}

function deserializeLabeledStatement(pos) {
  let node = {
    type: "LabeledStatement",
    label: null,
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.label = deserializeLabelIdentifier(pos + 8);
  node.body = deserializeStatement(pos + 32);
  return node;
}

function deserializeThrowStatement(pos) {
  let node = {
    type: "ThrowStatement",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeExpression(pos + 8);
  return node;
}

function deserializeTryStatement(pos) {
  let node = {
    type: "TryStatement",
    block: null,
    handler: null,
    finalizer: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.block = deserializeBoxBlockStatement(pos + 8);
  node.handler = deserializeOptionBoxCatchClause(pos + 16);
  node.finalizer = deserializeOptionBoxBlockStatement(pos + 24);
  return node;
}

function deserializeCatchClause(pos) {
  let node = {
    type: "CatchClause",
    param: null,
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.param = deserializeOptionCatchParameter(pos + 8);
  node.body = deserializeBoxBlockStatement(pos + 48);
  return node;
}

function deserializeCatchParameter(pos) {
  return deserializeBindingPattern(pos + 8);
}

function deserializeDebuggerStatement(pos) {
  return {
    type: "DebuggerStatement",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeBindingPattern(pos) {
  return deserializeBindingPatternKind(pos);
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
  let node = {
    type: "AssignmentPattern",
    left: null,
    right: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.left = deserializeBindingPattern(pos + 8);
  node.right = deserializeExpression(pos + 40);
  return node;
}

function deserializeObjectPattern(pos) {
  let node = {
      type: "ObjectPattern",
      properties: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    properties = deserializeVecBindingProperty(pos + 8),
    rest = deserializeOptionBoxBindingRestElement(pos + 32);
  rest !== null && properties.push(rest);
  node.properties = properties;
  return node;
}

function deserializeBindingProperty(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Property",
      kind: null,
      key: null,
      value: null,
      method: null,
      shorthand: deserializeBool(pos + 56),
      computed: deserializeBool(pos + 57),
      start,
      end,
    };
  node.kind = "init";
  node.key = deserializePropertyKey(pos + 8);
  node.value = deserializeBindingPattern(pos + 24);
  node.method = false;
  return node;
}

function deserializeArrayPattern(pos) {
  let node = {
      type: "ArrayPattern",
      elements: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    elements = deserializeVecOptionBindingPattern(pos + 8),
    rest = deserializeOptionBoxBindingRestElement(pos + 32);
  rest !== null && elements.push(rest);
  node.elements = elements;
  return node;
}

function deserializeBindingRestElement(pos) {
  let node = {
    type: "RestElement",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeBindingPattern(pos + 8);
  return node;
}

function deserializeFunction(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: deserializeFunctionType(pos + 84),
      id: null,
      generator: deserializeBool(pos + 85),
      async: deserializeBool(pos + 86),
      params: null,
      body: null,
      expression: null,
      start,
      end,
    },
    params = deserializeBoxFormalParameters(pos + 56);
  node.id = deserializeOptionBindingIdentifier(pos + 8);
  node.params = params;
  node.body = deserializeOptionBoxFunctionBody(pos + 72);
  node.expression = false;
  return node;
}

function deserializeFunctionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return "FunctionDeclaration";
    case 1:
      return "FunctionExpression";
    case 2:
      return "TSDeclareFunction";
    case 3:
      return "TSEmptyBodyFunctionExpression";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for FunctionType`);
  }
}

function deserializeFormalParameters(pos) {
  let params = deserializeVecFormalParameter(pos + 8);
  if (uint32[(pos + 32) >> 2] !== 0 && uint32[(pos + 36) >> 2] !== 0) {
    pos = uint32[(pos + 32) >> 2];
    let rest = {
      type: "RestElement",
      argument: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    };
    rest.argument = deserializeBindingPatternKind(pos + 8);
    params.push(rest);
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param;
  param = deserializeBindingPatternKind(pos + 32);
  return param;
}

function deserializeFunctionBody(pos) {
  let node = {
      type: "BlockStatement",
      body: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 32));
  node.body = body;
  return node;
}

function deserializeArrowFunctionExpression(pos) {
  let expression = deserializeBool(pos + 44),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "ArrowFunctionExpression",
      expression,
      async: deserializeBool(pos + 45),
      params: null,
      body: null,
      id: null,
      generator: null,
      start,
      end,
    },
    body = deserializeBoxFunctionBody(pos + 32);
  expression === true && (body = body.body[0].expression);
  node.params = deserializeBoxFormalParameters(pos + 16);
  node.body = body;
  node.generator = false;
  return node;
}

function deserializeYieldExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "YieldExpression",
      delegate: deserializeBool(pos + 24),
      argument: null,
      start,
      end,
    };
  node.argument = deserializeOptionExpression(pos + 8);
  return node;
}

function deserializeClass(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: deserializeClassType(pos + 132),
      decorators: null,
      id: null,
      superClass: null,
      body: null,
      start,
      end,
    };
  node.decorators = deserializeVecDecorator(pos + 8);
  node.id = deserializeOptionBindingIdentifier(pos + 32);
  node.superClass = deserializeOptionExpression(pos + 72);
  node.body = deserializeBoxClassBody(pos + 120);
  return node;
}

function deserializeClassType(pos) {
  switch (uint8[pos]) {
    case 0:
      return "ClassDeclaration";
    case 1:
      return "ClassExpression";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ClassType`);
  }
}

function deserializeClassBody(pos) {
  let node = {
    type: "ClassBody",
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.body = deserializeVecClassElement(pos + 8);
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: deserializeMethodDefinitionType(pos + 56),
      decorators: null,
      key: null,
      value: null,
      kind: deserializeMethodDefinitionKind(pos + 57),
      computed: deserializeBool(pos + 58),
      static: deserializeBool(pos + 59),
      start,
      end,
    };
  node.decorators = deserializeVecDecorator(pos + 8);
  node.key = deserializePropertyKey(pos + 32);
  node.value = deserializeBoxFunction(pos + 48);
  return node;
}

function deserializeMethodDefinitionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return "MethodDefinition";
    case 1:
      return "TSAbstractMethodDefinition";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for MethodDefinitionType`);
  }
}

function deserializePropertyDefinition(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: deserializePropertyDefinitionType(pos + 72),
      decorators: null,
      key: null,
      value: null,
      computed: deserializeBool(pos + 73),
      static: deserializeBool(pos + 74),
      start,
      end,
    };
  node.decorators = deserializeVecDecorator(pos + 8);
  node.key = deserializePropertyKey(pos + 32);
  node.value = deserializeOptionExpression(pos + 56);
  return node;
}

function deserializePropertyDefinitionType(pos) {
  switch (uint8[pos]) {
    case 0:
      return "PropertyDefinition";
    case 1:
      return "TSAbstractPropertyDefinition";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for PropertyDefinitionType`);
  }
}

function deserializeMethodDefinitionKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "constructor";
    case 1:
      return "method";
    case 2:
      return "get";
    case 3:
      return "set";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for MethodDefinitionKind`);
  }
}

function deserializePrivateIdentifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "PrivateIdentifier",
    name: deserializeStr(pos + 8),
    start,
    end,
  };
}

function deserializeStaticBlock(pos) {
  let node = {
    type: "StaticBlock",
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.body = deserializeVecStatement(pos + 8);
  return node;
}

function deserializeAccessorPropertyType(pos) {
  switch (uint8[pos]) {
    case 0:
      return "AccessorProperty";
    case 1:
      return "TSAbstractAccessorProperty";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AccessorPropertyType`);
  }
}

function deserializeAccessorProperty(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: deserializeAccessorPropertyType(pos + 72),
      decorators: null,
      key: null,
      value: null,
      computed: deserializeBool(pos + 73),
      static: deserializeBool(pos + 74),
      start,
      end,
    };
  node.decorators = deserializeVecDecorator(pos + 8);
  node.key = deserializePropertyKey(pos + 32);
  node.value = deserializeOptionExpression(pos + 56);
  return node;
}

function deserializeImportExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "ImportExpression",
      source: null,
      options: null,
      phase: deserializeOptionImportPhase(pos + 40),
      start,
      end,
    };
  node.source = deserializeExpression(pos + 8);
  node.options = deserializeOptionExpression(pos + 24);
  return node;
}

function deserializeImportDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "ImportDeclaration",
      specifiers: null,
      source: null,
      phase: deserializeOptionImportPhase(pos + 88),
      attributes: null,
      start,
      end,
    },
    specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 8);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 80);
  node.specifiers = specifiers;
  node.source = deserializeStringLiteral(pos + 32);
  node.attributes = withClause === null ? [] : withClause.attributes;
  return node;
}

function deserializeImportPhase(pos) {
  switch (uint8[pos]) {
    case 0:
      return "source";
    case 1:
      return "defer";
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
  let node = {
    type: "ImportSpecifier",
    imported: null,
    local: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.imported = deserializeModuleExportName(pos + 8);
  node.local = deserializeBindingIdentifier(pos + 64);
  return node;
}

function deserializeImportDefaultSpecifier(pos) {
  let node = {
    type: "ImportDefaultSpecifier",
    local: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.local = deserializeBindingIdentifier(pos + 8);
  return node;
}

function deserializeImportNamespaceSpecifier(pos) {
  let node = {
    type: "ImportNamespaceSpecifier",
    local: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.local = deserializeBindingIdentifier(pos + 8);
  return node;
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 8) };
}

function deserializeImportAttribute(pos) {
  let node = {
    type: "ImportAttribute",
    key: null,
    value: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.key = deserializeImportAttributeKey(pos + 8);
  node.value = deserializeStringLiteral(pos + 64);
  return node;
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
  let node = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      attributes: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    withClause = deserializeOptionBoxWithClause(pos + 96);
  node.declaration = deserializeOptionDeclaration(pos + 8);
  node.specifiers = deserializeVecExportSpecifier(pos + 24);
  node.source = deserializeOptionStringLiteral(pos + 48);
  node.attributes = withClause === null ? [] : withClause.attributes;
  return node;
}

function deserializeExportDefaultDeclaration(pos) {
  let node = {
    type: "ExportDefaultDeclaration",
    declaration: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.declaration = deserializeExportDefaultDeclarationKind(pos + 8);
  return node;
}

function deserializeExportAllDeclaration(pos) {
  let node = {
      type: "ExportAllDeclaration",
      exported: null,
      source: null,
      attributes: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    withClause = deserializeOptionBoxWithClause(pos + 112);
  node.exported = deserializeOptionModuleExportName(pos + 8);
  node.source = deserializeStringLiteral(pos + 64);
  node.attributes = withClause === null ? [] : withClause.attributes;
  return node;
}

function deserializeExportSpecifier(pos) {
  let node = {
    type: "ExportSpecifier",
    local: null,
    exported: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.local = deserializeModuleExportName(pos + 8);
  node.exported = deserializeModuleExportName(pos + 64);
  return node;
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
  let node = {
    type: "V8IntrinsicExpression",
    name: null,
    arguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.name = deserializeIdentifierName(pos + 8);
  node.arguments = deserializeVecArgument(pos + 32);
  return node;
}

function deserializeBooleanLiteral(pos) {
  let value = deserializeBool(pos + 8),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Literal",
      value,
      raw: null,
      start,
      end,
    };
  node.raw = start === 0 && end === 0 ? null : value + "";
  return node;
}

function deserializeNullLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Literal",
      value: null,
      raw: null,
      start,
      end,
    };
  node.raw = start === 0 && end === 0 ? null : "null";
  return node;
}

function deserializeNumericLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Literal",
    value: deserializeF64(pos + 8),
    raw: deserializeOptionStr(pos + 16),
    start,
    end,
  };
}

function deserializeStringLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Literal",
      value: null,
      raw: deserializeOptionStr(pos + 24),
      start,
      end,
    },
    value = deserializeStr(pos + 8);
  deserializeBool(pos + 40) &&
    (value = value.replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16))));
  node.value = value;
  return node;
}

function deserializeBigIntLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Literal",
      value: null,
      raw: deserializeOptionStr(pos + 24),
      bigint: null,
      start,
      end,
    },
    bigint = deserializeStr(pos + 8);
  node.value = BigInt(bigint);
  node.bigint = bigint;
  return node;
}

function deserializeRegExpLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Literal",
      value: null,
      raw: deserializeOptionStr(pos + 40),
      regex: null,
      start,
      end,
    },
    regex = deserializeRegExp(pos + 8),
    value = null;
  try {
    value = new RegExp(regex.pattern, regex.flags);
  } catch {}
  node.value = value;
  node.regex = regex;
  return node;
}

function deserializeRegExp(pos) {
  return {
    pattern: deserializeStr(pos),
    flags: deserializeRegExpFlags(pos + 24),
  };
}

function deserializeRegExpFlags(pos) {
  let flagBits = deserializeU8(pos),
    flags = "";
  // Alphabetical order
  flagBits & 64 && (flags += "d");
  flagBits & 1 && (flags += "g");
  flagBits & 2 && (flags += "i");
  flagBits & 4 && (flags += "m");
  flagBits & 8 && (flags += "s");
  flagBits & 16 && (flags += "u");
  flagBits & 128 && (flags += "v");
  flagBits & 32 && (flags += "y");
  return flags;
}

function deserializeJSXElement(pos) {
  let node = {
      type: "JSXElement",
      openingElement: null,
      children: null,
      closingElement: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    closingElement = deserializeOptionBoxJSXClosingElement(pos + 40),
    openingElement = deserializeBoxJSXOpeningElement(pos + 8);
  closingElement === null && (openingElement.selfClosing = true);
  node.openingElement = openingElement;
  node.children = deserializeVecJSXChild(pos + 16);
  node.closingElement = closingElement;
  return node;
}

function deserializeJSXOpeningElement(pos) {
  let node = {
    type: "JSXOpeningElement",
    name: null,
    attributes: null,
    selfClosing: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.name = deserializeJSXElementName(pos + 8);
  node.attributes = deserializeVecJSXAttributeItem(pos + 32);
  node.selfClosing = false;
  return node;
}

function deserializeJSXClosingElement(pos) {
  let node = {
    type: "JSXClosingElement",
    name: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.name = deserializeJSXElementName(pos + 8);
  return node;
}

function deserializeJSXFragment(pos) {
  let node = {
    type: "JSXFragment",
    openingFragment: null,
    children: null,
    closingFragment: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.openingFragment = deserializeJSXOpeningFragment(pos + 8);
  node.children = deserializeVecJSXChild(pos + 16);
  node.closingFragment = deserializeJSXClosingFragment(pos + 40);
  return node;
}

function deserializeJSXOpeningFragment(pos) {
  let node = {
    type: "JSXOpeningFragment",
    attributes: null,
    selfClosing: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.attributes = [];
  node.selfClosing = false;
  return node;
}

function deserializeJSXClosingFragment(pos) {
  return {
    type: "JSXClosingFragment",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeJSXElementName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxJSXIdentifier(pos + 8);
    case 1:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return {
        type: "JSXIdentifier",
        name: ident.name,
        start: ident.start,
        end: ident.end,
      };
    case 2:
      return deserializeBoxJSXNamespacedName(pos + 8);
    case 3:
      return deserializeBoxJSXMemberExpression(pos + 8);
    case 4:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return {
        type: "JSXIdentifier",
        name: "this",
        start: thisExpr.start,
        end: thisExpr.end,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  let node = {
    type: "JSXNamespacedName",
    namespace: null,
    name: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.namespace = deserializeJSXIdentifier(pos + 8);
  node.name = deserializeJSXIdentifier(pos + 32);
  return node;
}

function deserializeJSXMemberExpression(pos) {
  let node = {
    type: "JSXMemberExpression",
    object: null,
    property: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.object = deserializeJSXMemberExpressionObject(pos + 8);
  node.property = deserializeJSXIdentifier(pos + 24);
  return node;
}

function deserializeJSXMemberExpressionObject(pos) {
  switch (uint8[pos]) {
    case 0:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return {
        type: "JSXIdentifier",
        name: ident.name,
        start: ident.start,
        end: ident.end,
      };
    case 1:
      return deserializeBoxJSXMemberExpression(pos + 8);
    case 2:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return {
        type: "JSXIdentifier",
        name: "this",
        start: thisExpr.start,
        end: thisExpr.end,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  let node = {
    type: "JSXExpressionContainer",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeJSXExpression(pos + 8);
  return node;
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
  return {
    type: "JSXEmptyExpression",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
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
  let node = {
    type: "JSXAttribute",
    name: null,
    value: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.name = deserializeJSXAttributeName(pos + 8);
  node.value = deserializeOptionJSXAttributeValue(pos + 24);
  return node;
}

function deserializeJSXSpreadAttribute(pos) {
  let node = {
    type: "JSXSpreadAttribute",
    argument: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.argument = deserializeExpression(pos + 8);
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "JSXIdentifier",
    name: deserializeStr(pos + 8),
    start,
    end,
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
  let node = {
    type: "JSXSpreadChild",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  return node;
}

function deserializeJSXText(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "JSXText",
    value: deserializeStr(pos + 8),
    raw: deserializeOptionStr(pos + 24),
    start,
    end,
  };
}

function deserializeTSThisParameter(pos) {
  let node = {
    type: "Identifier",
    decorators: null,
    name: null,
    optional: null,
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.decorators = [];
  node.name = "this";
  node.optional = false;
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 16);
  return node;
}

function deserializeTSEnumDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSEnumDeclaration",
      id: null,
      body: null,
      const: deserializeBool(pos + 76),
      declare: deserializeBool(pos + 77),
      start,
      end,
    };
  node.id = deserializeBindingIdentifier(pos + 8);
  node.body = deserializeTSEnumBody(pos + 40);
  return node;
}

function deserializeTSEnumBody(pos) {
  let node = {
    type: "TSEnumBody",
    members: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.members = deserializeVecTSEnumMember(pos + 8);
  return node;
}

function deserializeTSEnumMember(pos) {
  let node = {
    type: "TSEnumMember",
    id: null,
    initializer: null,
    computed: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.id = deserializeTSEnumMemberName(pos + 8);
  node.initializer = deserializeOptionExpression(pos + 24);
  node.computed = deserializeU8(pos + 8) > 1;
  return node;
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
  let node = {
    type: "TSTypeAnnotation",
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
}

function deserializeTSLiteralType(pos) {
  let node = {
    type: "TSLiteralType",
    literal: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.literal = deserializeTSLiteral(pos + 8);
  return node;
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
  let node = {
    type: "TSConditionalType",
    checkType: null,
    extendsType: null,
    trueType: null,
    falseType: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.checkType = deserializeTSType(pos + 8);
  node.extendsType = deserializeTSType(pos + 24);
  node.trueType = deserializeTSType(pos + 40);
  node.falseType = deserializeTSType(pos + 56);
  return node;
}

function deserializeTSUnionType(pos) {
  let node = {
    type: "TSUnionType",
    types: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.types = deserializeVecTSType(pos + 8);
  return node;
}

function deserializeTSIntersectionType(pos) {
  let node = {
    type: "TSIntersectionType",
    types: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.types = deserializeVecTSType(pos + 8);
  return node;
}

function deserializeTSParenthesizedType(pos) {
  let node;
  node = {
    type: "TSParenthesizedType",
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
}

function deserializeTSTypeOperator(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSTypeOperator",
      operator: deserializeTSTypeOperatorOperator(pos + 24),
      typeAnnotation: null,
      start,
      end,
    };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
}

function deserializeTSTypeOperatorOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return "keyof";
    case 1:
      return "unique";
    case 2:
      return "readonly";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeOperatorOperator`);
  }
}

function deserializeTSArrayType(pos) {
  let node = {
    type: "TSArrayType",
    elementType: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.elementType = deserializeTSType(pos + 8);
  return node;
}

function deserializeTSIndexedAccessType(pos) {
  let node = {
    type: "TSIndexedAccessType",
    objectType: null,
    indexType: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.objectType = deserializeTSType(pos + 8);
  node.indexType = deserializeTSType(pos + 24);
  return node;
}

function deserializeTSTupleType(pos) {
  let node = {
    type: "TSTupleType",
    elementTypes: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.elementTypes = deserializeVecTSTupleElement(pos + 8);
  return node;
}

function deserializeTSNamedTupleMember(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSNamedTupleMember",
      label: null,
      elementType: null,
      optional: deserializeBool(pos + 48),
      start,
      end,
    };
  node.label = deserializeIdentifierName(pos + 8);
  node.elementType = deserializeTSTupleElement(pos + 32);
  return node;
}

function deserializeTSOptionalType(pos) {
  let node = {
    type: "TSOptionalType",
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
}

function deserializeTSRestType(pos) {
  let node = {
    type: "TSRestType",
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
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
  return {
    type: "TSAnyKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSStringKeyword(pos) {
  return {
    type: "TSStringKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSBooleanKeyword(pos) {
  return {
    type: "TSBooleanKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSNumberKeyword(pos) {
  return {
    type: "TSNumberKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSNeverKeyword(pos) {
  return {
    type: "TSNeverKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  return {
    type: "TSIntrinsicKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSUnknownKeyword(pos) {
  return {
    type: "TSUnknownKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSNullKeyword(pos) {
  return {
    type: "TSNullKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSUndefinedKeyword(pos) {
  return {
    type: "TSUndefinedKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSVoidKeyword(pos) {
  return {
    type: "TSVoidKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSSymbolKeyword(pos) {
  return {
    type: "TSSymbolKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSThisType(pos) {
  return {
    type: "TSThisType",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSObjectKeyword(pos) {
  return {
    type: "TSObjectKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSBigIntKeyword(pos) {
  return {
    type: "TSBigIntKeyword",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeTSTypeReference(pos) {
  let node = {
    type: "TSTypeReference",
    typeName: null,
    typeArguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeName = deserializeTSTypeName(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  return node;
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
  let node = {
    type: "TSQualifiedName",
    left: null,
    right: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.left = deserializeTSTypeName(pos + 8);
  node.right = deserializeIdentifierName(pos + 24);
  return node;
}

function deserializeTSTypeParameterInstantiation(pos) {
  let node = {
    type: "TSTypeParameterInstantiation",
    params: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.params = deserializeVecTSType(pos + 8);
  return node;
}

function deserializeTSTypeParameter(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSTypeParameter",
      name: null,
      constraint: null,
      default: null,
      in: deserializeBool(pos + 72),
      out: deserializeBool(pos + 73),
      const: deserializeBool(pos + 74),
      start,
      end,
    };
  node.name = deserializeBindingIdentifier(pos + 8);
  node.constraint = deserializeOptionTSType(pos + 40);
  node.default = deserializeOptionTSType(pos + 56);
  return node;
}

function deserializeTSTypeParameterDeclaration(pos) {
  let node = {
    type: "TSTypeParameterDeclaration",
    params: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.params = deserializeVecTSTypeParameter(pos + 8);
  return node;
}

function deserializeTSTypeAliasDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSTypeAliasDeclaration",
      id: null,
      typeParameters: null,
      typeAnnotation: null,
      declare: deserializeBool(pos + 68),
      start,
      end,
    };
  node.id = deserializeBindingIdentifier(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 40);
  node.typeAnnotation = deserializeTSType(pos + 48);
  return node;
}

function deserializeTSInterfaceDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSInterfaceDeclaration",
      id: null,
      typeParameters: null,
      extends: null,
      body: null,
      declare: deserializeBool(pos + 84),
      start,
      end,
    };
  node.id = deserializeBindingIdentifier(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 40);
  node.extends = deserializeVecTSInterfaceHeritage(pos + 48);
  node.body = deserializeBoxTSInterfaceBody(pos + 72);
  return node;
}

function deserializeTSInterfaceBody(pos) {
  let node = {
    type: "TSInterfaceBody",
    body: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.body = deserializeVecTSSignature(pos + 8);
  return node;
}

function deserializeTSPropertySignature(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSPropertySignature",
      computed: deserializeBool(pos + 32),
      optional: deserializeBool(pos + 33),
      readonly: deserializeBool(pos + 34),
      key: null,
      typeAnnotation: null,
      accessibility: null,
      static: null,
      start,
      end,
    };
  node.key = deserializePropertyKey(pos + 8);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  node.static = false;
  return node;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSIndexSignature",
      parameters: null,
      typeAnnotation: null,
      readonly: deserializeBool(pos + 40),
      static: deserializeBool(pos + 41),
      accessibility: null,
      start,
      end,
    };
  node.parameters = deserializeVecTSIndexSignatureName(pos + 8);
  node.typeAnnotation = deserializeBoxTSTypeAnnotation(pos + 32);
  return node;
}

function deserializeTSCallSignatureDeclaration(pos) {
  let node = {
      type: "TSCallSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    params = deserializeBoxFormalParameters(pos + 24),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 16);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  return node;
}

function deserializeTSMethodSignatureKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "method";
    case 1:
      return "get";
    case 2:
      return "set";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSMethodSignatureKind`);
  }
}

function deserializeTSMethodSignature(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSMethodSignature",
      key: null,
      computed: deserializeBool(pos + 60),
      optional: deserializeBool(pos + 61),
      kind: deserializeTSMethodSignatureKind(pos + 62),
      typeParameters: null,
      params: null,
      returnType: null,
      accessibility: null,
      readonly: null,
      static: null,
      start,
      end,
    },
    params = deserializeBoxFormalParameters(pos + 40),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 32);
  thisParam !== null && params.unshift(thisParam);
  node.key = deserializePropertyKey(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 24);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 48);
  node.readonly = false;
  node.static = false;
  return node;
}

function deserializeTSConstructSignatureDeclaration(pos) {
  let node = {
    type: "TSConstructSignatureDeclaration",
    typeParameters: null,
    params: null,
    returnType: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = deserializeBoxFormalParameters(pos + 16);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  return node;
}

function deserializeTSIndexSignatureName(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "Identifier",
      decorators: null,
      name: deserializeStr(pos + 8),
      optional: null,
      typeAnnotation: null,
      start,
      end,
    };
  node.decorators = [];
  node.optional = false;
  node.typeAnnotation = deserializeBoxTSTypeAnnotation(pos + 24);
  return node;
}

function deserializeTSInterfaceHeritage(pos) {
  let node = {
    type: "TSInterfaceHeritage",
    expression: null,
    typeArguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  return node;
}

function deserializeTSTypePredicate(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSTypePredicate",
      parameterName: null,
      asserts: deserializeBool(pos + 32),
      typeAnnotation: null,
      start,
      end,
    };
  node.parameterName = deserializeTSTypePredicateName(pos + 8);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  return node;
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
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    declare = deserializeBool(pos + 85),
    node,
    body = deserializeOptionTSModuleDeclarationBody(pos + 64);
  if (body === null) {
    node = {
      type: "TSModuleDeclaration",
      id: null,
      kind,
      declare,
      global: false,
      start,
      end,
    };
    node.id = deserializeTSModuleDeclarationName(pos + 8);
  } else {
    node = {
      type: "TSModuleDeclaration",
      id: null,
      body,
      kind,
      declare,
      global: false,
      start,
      end,
    };
    let id = deserializeTSModuleDeclarationName(pos + 8);
    if (body.type === "TSModuleBlock") node.id = id;
    else {
      let innerId = body.id;
      if (innerId.type === "Identifier")
        node.id = {
          type: "TSQualifiedName",
          left: id,
          right: innerId,
          start: id.start,
          end: innerId.end,
        };
      else {
        // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with `id` of
        // this module on left, and previous `left` of innermost `TSQualifiedName` on right
        node.id = innerId;
        let { start } = id;
        for (;;) {
          innerId.start = start;
          if (innerId.left.type === "Identifier") break;
          innerId = innerId.left;
        }
        let right = innerId.left;
        innerId.left = {
          type: "TSQualifiedName",
          left: id,
          right,
          start,
          end: right.end,
        };
      }
      if (Object.hasOwn(body, "body")) {
        body = body.body;
        node.body = body;
      } else body = null;
    }
  }
  return node;
}

function deserializeTSModuleDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "module";
    case 1:
      return "namespace";
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

function deserializeTSGlobalDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSModuleDeclaration",
      id: null,
      body: null,
      kind: null,
      declare: deserializeBool(pos + 76),
      global: null,
      start,
      end,
    };
  node.id = {
    type: "Identifier",
    name: "global",
    start: deserializeU32(pos + 8),
    end: deserializeU32(pos + 12),
  };
  node.body = deserializeTSModuleBlock(pos + 16);
  node.kind = "global";
  node.global = true;
  return node;
}

function deserializeTSModuleBlock(pos) {
  let node = {
      type: "TSModuleBlock",
      body: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 32));
  node.body = body;
  return node;
}

function deserializeTSTypeLiteral(pos) {
  let node = {
    type: "TSTypeLiteral",
    members: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.members = deserializeVecTSSignature(pos + 8);
  return node;
}

function deserializeTSInferType(pos) {
  let node = {
    type: "TSInferType",
    typeParameter: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeParameter = deserializeBoxTSTypeParameter(pos + 8);
  return node;
}

function deserializeTSTypeQuery(pos) {
  let node = {
    type: "TSTypeQuery",
    exprName: null,
    typeArguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.exprName = deserializeTSTypeQueryExprName(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  return node;
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
  let node = {
    type: "TSImportType",
    source: null,
    options: null,
    qualifier: null,
    typeArguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.source = deserializeStringLiteral(pos + 8);
  node.options = deserializeOptionBoxObjectExpression(pos + 56);
  node.qualifier = deserializeOptionTSImportTypeQualifier(pos + 64);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 80);
  return node;
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
  let node = {
    type: "TSQualifiedName",
    left: null,
    right: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.left = deserializeTSImportTypeQualifier(pos + 8);
  node.right = deserializeIdentifierName(pos + 24);
  return node;
}

function deserializeTSFunctionType(pos) {
  let node = {
      type: "TSFunctionType",
      typeParameters: null,
      params: null,
      returnType: null,
      start: deserializeU32(pos),
      end: deserializeU32(pos + 4),
    },
    params = deserializeBoxFormalParameters(pos + 24),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 16);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = params;
  node.returnType = deserializeBoxTSTypeAnnotation(pos + 32);
  return node;
}

function deserializeTSConstructorType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSConstructorType",
      abstract: deserializeBool(pos + 36),
      typeParameters: null,
      params: null,
      returnType: null,
      start,
      end,
    };
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = deserializeBoxFormalParameters(pos + 16);
  node.returnType = deserializeBoxTSTypeAnnotation(pos + 24);
  return node;
}

function deserializeTSMappedType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSMappedType",
      key: null,
      constraint: null,
      nameType: null,
      typeAnnotation: null,
      optional: null,
      readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 53),
      start,
      end,
    },
    typeParameter = deserializeBoxTSTypeParameter(pos + 8),
    key = typeParameter.name,
    { constraint } = typeParameter,
    optional = deserializeOptionTSMappedTypeModifierOperator(pos + 52);
  optional === null && (optional = false);
  node.key = key;
  node.constraint = constraint;
  node.nameType = deserializeOptionTSType(pos + 16);
  node.typeAnnotation = deserializeOptionTSType(pos + 32);
  node.optional = optional;
  return node;
}

function deserializeTSMappedTypeModifierOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return true;
    case 1:
      return "+";
    case 2:
      return "-";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSMappedTypeModifierOperator`);
  }
}

function deserializeTSTemplateLiteralType(pos) {
  let node = {
    type: "TSTemplateLiteralType",
    quasis: null,
    types: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.quasis = deserializeVecTemplateElement(pos + 8);
  node.types = deserializeVecTSType(pos + 32);
  return node;
}

function deserializeTSAsExpression(pos) {
  let node = {
    type: "TSAsExpression",
    expression: null,
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  node.typeAnnotation = deserializeTSType(pos + 24);
  return node;
}

function deserializeTSSatisfiesExpression(pos) {
  let node = {
    type: "TSSatisfiesExpression",
    expression: null,
    typeAnnotation: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  node.typeAnnotation = deserializeTSType(pos + 24);
  return node;
}

function deserializeTSTypeAssertion(pos) {
  let node = {
    type: "TSTypeAssertion",
    typeAnnotation: null,
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.typeAnnotation = deserializeTSType(pos + 8);
  node.expression = deserializeExpression(pos + 24);
  return node;
}

function deserializeTSImportEqualsDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSImportEqualsDeclaration",
      id: null,
      moduleReference: null,
      importKind: deserializeImportOrExportKind(pos + 56),
      start,
      end,
    };
  node.id = deserializeBindingIdentifier(pos + 8);
  node.moduleReference = deserializeTSModuleReference(pos + 40);
  return node;
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
  let node = {
    type: "TSExternalModuleReference",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeStringLiteral(pos + 8);
  return node;
}

function deserializeTSNonNullExpression(pos) {
  let node = {
    type: "TSNonNullExpression",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  return node;
}

function deserializeDecorator(pos) {
  let node = {
    type: "Decorator",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  return node;
}

function deserializeTSExportAssignment(pos) {
  let node = {
    type: "TSExportAssignment",
    expression: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  return node;
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let node = {
    type: "TSNamespaceExportDeclaration",
    id: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.id = deserializeIdentifierName(pos + 8);
  return node;
}

function deserializeTSInstantiationExpression(pos) {
  let node = {
    type: "TSInstantiationExpression",
    expression: null,
    typeArguments: null,
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 8);
  node.typeArguments = deserializeBoxTSTypeParameterInstantiation(pos + 24);
  return node;
}

function deserializeImportOrExportKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "value";
    case 1:
      return "type";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportOrExportKind`);
  }
}

function deserializeJSDocNullableType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSJSDocNullableType",
      typeAnnotation: null,
      postfix: deserializeBool(pos + 24),
      start,
      end,
    };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
}

function deserializeJSDocNonNullableType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    node = {
      type: "TSJSDocNonNullableType",
      typeAnnotation: null,
      postfix: deserializeBool(pos + 24),
      start,
      end,
    };
  node.typeAnnotation = deserializeTSType(pos + 8);
  return node;
}

function deserializeJSDocUnknownType(pos) {
  return {
    type: "TSJSDocUnknownType",
    start: deserializeU32(pos),
    end: deserializeU32(pos + 4),
  };
}

function deserializeCommentKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "Line";
    case 1:
      return "Block";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for CommentKind`);
  }
}

function deserializeComment(pos) {
  let type = deserializeCommentKind(pos + 12),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type,
    value: sourceText.slice(start + 2, end - (type === "Line" ? 0 : 2)),
    start,
    end,
  };
}

function deserializeNameSpan(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    value: deserializeStr(pos + 8),
    start,
    end,
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
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      return {
        kind: "NamespaceObject",
        name: null,
        start: null,
        end: null,
      };
    case 2:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: "Default",
        name: null,
        start,
        end,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportImportName`);
  }
}

function deserializeExportEntry(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeOptionNameSpan(pos + 16),
    importName: deserializeExportImportName(pos + 40),
    exportName: deserializeExportExportName(pos + 72),
    localName: deserializeExportLocalName(pos + 104),
    isType: deserializeBool(pos + 136),
    start,
    end,
  };
}

function deserializeExportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      return {
        kind: "All",
        name: null,
        start: null,
        end: null,
      };
    case 2:
      return {
        kind: "AllButDefault",
        name: null,
        start: null,
        end: null,
      };
    case 3:
      return {
        kind: "None",
        name: null,
        start: null,
        end: null,
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
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: "Default",
        name: null,
        start,
        end,
      };
    case 2:
      return {
        kind: "None",
        name: null,
        start: null,
        end: null,
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
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Default",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 2:
      return {
        kind: "None",
        name: null,
        start: null,
        end: null,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportLocalName`);
  }
}

function deserializeDynamicImport(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeSpan(pos + 8),
    start,
    end,
  };
}

function deserializeAssignmentOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return "=";
    case 1:
      return "+=";
    case 2:
      return "-=";
    case 3:
      return "*=";
    case 4:
      return "/=";
    case 5:
      return "%=";
    case 6:
      return "**=";
    case 7:
      return "<<=";
    case 8:
      return ">>=";
    case 9:
      return ">>>=";
    case 10:
      return "|=";
    case 11:
      return "^=";
    case 12:
      return "&=";
    case 13:
      return "||=";
    case 14:
      return "&&=";
    case 15:
      return "??=";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentOperator`);
  }
}

function deserializeBinaryOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return "==";
    case 1:
      return "!=";
    case 2:
      return "===";
    case 3:
      return "!==";
    case 4:
      return "<";
    case 5:
      return "<=";
    case 6:
      return ">";
    case 7:
      return ">=";
    case 8:
      return "+";
    case 9:
      return "-";
    case 10:
      return "*";
    case 11:
      return "/";
    case 12:
      return "%";
    case 13:
      return "**";
    case 14:
      return "<<";
    case 15:
      return ">>";
    case 16:
      return ">>>";
    case 17:
      return "|";
    case 18:
      return "^";
    case 19:
      return "&";
    case 20:
      return "in";
    case 21:
      return "instanceof";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for BinaryOperator`);
  }
}

function deserializeLogicalOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return "||";
    case 1:
      return "&&";
    case 2:
      return "??";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for LogicalOperator`);
  }
}

function deserializeUnaryOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return "+";
    case 1:
      return "-";
    case 2:
      return "!";
    case 3:
      return "~";
    case 4:
      return "typeof";
    case 5:
      return "void";
    case 6:
      return "delete";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for UnaryOperator`);
  }
}

function deserializeUpdateOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return "++";
    case 1:
      return "--";
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
      return "script";
    case 1:
      return "module";
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
      return "Error";
    case 1:
      return "Warning";
    case 2:
      return "Advice";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ErrorSeverity`);
  }
}

function deserializeErrorLabel(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    message: deserializeOptionStr(pos + 8),
    start,
    end,
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeNameSpan(pos + 8),
    entries: deserializeVecImportEntry(pos + 32),
    start,
    end,
  };
}

function deserializeStaticExport(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    entries: deserializeVecExportEntry(pos + 8),
    start,
    end,
  };
}

function deserializeU32(pos) {
  return uint32[pos >> 2];
}

function deserializeU8(pos) {
  return uint8[pos];
}

function deserializeStr(pos) {
  let pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  if (len === 0) return "";
  pos = uint32[pos32];
  if (sourceIsAscii && pos < sourceByteLen) return sourceText.substr(pos, len);
  // Longer strings use `TextDecoder`
  // TODO: Find best switch-over point
  let end = pos + len;
  if (len > 50) return decodeStr(uint8.subarray(pos, end));
  // Shorter strings decode by hand to avoid native call
  let out = "",
    c;
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
    arr.push(deserializeComment(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionHashbang(pos) {
  if (uint32[(pos + 8) >> 2] === 0 && uint32[(pos + 12) >> 2] === 0) return null;
  return deserializeHashbang(pos);
}

function deserializeVecDirective(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 72;
  for (; pos !== endPos; ) {
    arr.push(deserializeDirective(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecStatement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
    arr.push(deserializeArrayExpressionElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxSpreadElement(pos) {
  return deserializeSpreadElement(uint32[pos >> 2]);
}

function deserializeVecObjectPropertyKind(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 48;
  for (; pos !== endPos; ) {
    arr.push(deserializeTemplateElement(pos));
    pos += 48;
  }
  return arr;
}

function deserializeVecExpression(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
    arr.push(deserializeOptionAssignmentTargetMaybeDefault(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxAssignmentTargetRest(pos) {
  return deserializeAssignmentTargetRest(uint32[pos >> 2]);
}

function deserializeOptionBoxAssignmentTargetRest(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxAssignmentTargetRest(pos);
}

function deserializeVecAssignmentTargetProperty(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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

function deserializeBoxTSGlobalDeclaration(pos) {
  return deserializeTSGlobalDeclaration(uint32[pos >> 2]);
}

function deserializeBoxTSImportEqualsDeclaration(pos) {
  return deserializeTSImportEqualsDeclaration(uint32[pos >> 2]);
}

function deserializeVecVariableDeclarator(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 64;
  for (; pos !== endPos; ) {
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
  if (uint32[(pos + 8) >> 2] === 0 && uint32[(pos + 12) >> 2] === 0) return null;
  return deserializeLabelIdentifier(pos);
}

function deserializeVecSwitchCase(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 48;
  for (; pos !== endPos; ) {
    arr.push(deserializeSwitchCase(pos));
    pos += 48;
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 64;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 72;
  for (; pos !== endPos; ) {
    arr.push(deserializeFormalParameter(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecDecorator(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 24;
  for (; pos !== endPos; ) {
    arr.push(deserializeDecorator(pos));
    pos += 24;
  }
  return arr;
}

function deserializeBoxClassBody(pos) {
  return deserializeClassBody(uint32[pos >> 2]);
}

function deserializeVecClassElement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 112;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 128;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
    arr.push(deserializeJSXChild(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxJSXClosingElement(pos) {
  return deserializeJSXClosingElement(uint32[pos >> 2]);
}

function deserializeOptionBoxJSXClosingElement(pos) {
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
  return deserializeBoxJSXClosingElement(pos);
}

function deserializeVecJSXAttributeItem(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 40;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSType(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecTSTupleElement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 80;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSTypeParameter(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecTSInterfaceHeritage(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSInterfaceHeritage(pos));
    pos += 32;
  }
  return arr;
}

function deserializeBoxTSInterfaceBody(pos) {
  return deserializeTSInterfaceBody(uint32[pos >> 2]);
}

function deserializeVecTSSignature(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
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
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos; ) {
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
  if (uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0) return null;
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
  if (uint32[(pos + 8) >> 2] === 0 && uint32[(pos + 12) >> 2] === 0) return null;
  return deserializeNameSpan(pos);
}

function deserializeVecError(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 80;
  for (; pos !== endPos; ) {
    arr.push(deserializeError(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecErrorLabel(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 24;
  for (; pos !== endPos; ) {
    arr.push(deserializeErrorLabel(pos));
    pos += 24;
  }
  return arr;
}

function deserializeVecStaticImport(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 56;
  for (; pos !== endPos; ) {
    arr.push(deserializeStaticImport(pos));
    pos += 56;
  }
  return arr;
}

function deserializeVecStaticExport(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos; ) {
    arr.push(deserializeStaticExport(pos));
    pos += 32;
  }
  return arr;
}

function deserializeVecDynamicImport(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 16;
  for (; pos !== endPos; ) {
    arr.push(deserializeDynamicImport(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecSpan(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 8;
  for (; pos !== endPos; ) {
    arr.push(deserializeSpan(pos));
    pos += 8;
  }
  return arr;
}

function deserializeVecImportEntry(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 96;
  for (; pos !== endPos; ) {
    arr.push(deserializeImportEntry(pos));
    pos += 96;
  }
  return arr;
}

function deserializeVecExportEntry(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 144;
  for (; pos !== endPos; ) {
    arr.push(deserializeExportEntry(pos));
    pos += 144;
  }
  return arr;
}
