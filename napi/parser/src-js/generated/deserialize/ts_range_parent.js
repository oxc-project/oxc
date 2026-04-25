// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8,
  int32,
  float64,
  sourceText,
  sourceTextLatin,
  sourceEndPos = 0,
  firstNonAsciiPos = 0,
  parent = null;

const { fromCharCode } = String,
  { utf8Slice, latin1Slice } = Buffer.prototype,
  stringDecodeArrays = Array(65).fill(null);
for (let i = 0; i <= 64; i++) stringDecodeArrays[i] = Array(i).fill(0);

export function deserialize(buffer, sourceText, sourceByteLen) {
  sourceEndPos = sourceByteLen;
  return deserializeWith(buffer, sourceText, sourceByteLen, deserializeRawTransferData);
}

function deserializeWith(buffer, sourceTextInput, sourceByteLen, deserialize) {
  uint8 = buffer;
  int32 = buffer.int32;
  float64 = buffer.float64;
  sourceText = sourceTextInput;
  if (sourceText.length === sourceByteLen) {
    firstNonAsciiPos = sourceByteLen;
    sourceTextLatin = sourceText;
  } else {
    let i = 0;
    for (; i < sourceByteLen && uint8[i] < 128; i++);
    firstNonAsciiPos = i;
    sourceTextLatin = latin1Slice.call(uint8, 0, sourceByteLen);
  }
  let data = deserialize(int32[536870900]);
  resetBuffer();
  return data;
}

export function resetBuffer() {
  // Clear buffer and source text strings to allow them to be garbage collected
  uint8 = int32 = float64 = sourceText = sourceTextLatin = void 0;
}

function deserializeProgram(pos) {
  let end = deserializeI32(pos + 4),
    program = (parent = {
      type: "Program",
      body: null,
      sourceType: deserializeModuleKind(pos + 137),
      hashbang: null,
      start: 0,
      end,
      range: [0, end],
      parent: null,
    });
  program.hashbang = deserializeOptionHashbang(pos + 56);
  let body = (program.body = deserializeVecDirective(pos + 88));
  body.push(...deserializeVecStatement(pos + 112));
  {
    let start;
    if (body.length > 0) {
      let first = body[0];
      start = first.start;
      if (first.type === "ExportNamedDeclaration" || first.type === "ExportDefaultDeclaration") {
        let { declaration } = first;
        if (
          declaration !== null &&
          declaration.type === "ClassDeclaration" &&
          declaration.decorators.length > 0
        ) {
          let decoratorStart = declaration.decorators[0].start;
          decoratorStart < start && (start = decoratorStart);
        }
      }
    } else start = end;
    program.start = program.range[0] = start;
  }
  parent = null;
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
  let start, end;
  return {
    type: "Identifier",
    decorators: [],
    name: deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeIdentifierReference(pos) {
  let start, end;
  return {
    type: "Identifier",
    decorators: [],
    name: deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeBindingIdentifier(pos) {
  let start, end;
  return {
    type: "Identifier",
    decorators: [],
    name: deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeLabelIdentifier(pos) {
  let start, end;
  return {
    type: "Identifier",
    decorators: [],
    name: deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeThisExpression(pos) {
  let start, end;
  return {
    type: "ThisExpression",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeArrayExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ArrayExpression",
      elements: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.elements = deserializeVecArrayExpressionElement(pos + 16);
  parent = previousParent;
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
      return deserializeBoxElision(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ArrayExpressionElement`);
  }
}

function deserializeElision(pos) {
  return null;
}

function deserializeObjectExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ObjectExpression",
      properties: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.properties = deserializeVecObjectPropertyKind(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: deserializePropertyKind(pos + 12),
      key: null,
      value: null,
      method: deserializeBool(pos + 13),
      shorthand: deserializeBool(pos + 14),
      computed: deserializeBool(pos + 15),
      optional: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeExpression(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TemplateLiteral",
      quasis: null,
      expressions: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.expressions = deserializeVecExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTaggedTemplateExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TaggedTemplateExpression",
      tag: null,
      typeArguments: null,
      quasi: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.tag = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.quasi = deserializeTemplateLiteral(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTemplateElement(pos) {
  let tail = deserializeBool(pos + 12),
    start = deserializeI32(pos) - 1,
    end = deserializeI32(pos + 4) + 2 - tail,
    value = deserializeTemplateElementValue(pos + 16);
  value.cooked !== null &&
    deserializeBool(pos + 13) &&
    (value.cooked = value.cooked.replace(/\uFFFD(.{4})/g, (_, hex) =>
      String.fromCodePoint(parseInt(hex, 16)),
    ));
  return {
    type: "TemplateElement",
    value,
    tail,
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 12),
      computed: true,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeStaticMemberExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 12),
      computed: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializePrivateFieldExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 12),
      computed: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.property = deserializePrivateIdentifier(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeCallExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "CallExpression",
      callee: null,
      typeArguments: null,
      arguments: null,
      optional: deserializeBool(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.callee = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.arguments = deserializeVecArgument(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeNewExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "NewExpression",
      callee: null,
      typeArguments: null,
      arguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.callee = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.arguments = deserializeVecArgument(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeMetaProperty(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "MetaProperty",
      meta: null,
      property: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.meta = deserializeIdentifierName(pos + 16);
  node.property = deserializeIdentifierName(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeSpreadElement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "SpreadElement",
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "UpdateExpression",
      operator: deserializeUpdateOperator(pos + 12),
      prefix: deserializeBool(pos + 13),
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeSimpleAssignmentTarget(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeUnaryExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "UnaryExpression",
      operator: deserializeUnaryOperator(pos + 12),
      argument: null,
      prefix: true,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBinaryExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "BinaryExpression",
      left: null,
      operator: deserializeBinaryOperator(pos + 12),
      right: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializePrivateInExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "BinaryExpression",
      left: null,
      operator: "in",
      right: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializePrivateIdentifier(pos + 16);
  node.right = deserializeExpression(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeLogicalExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "LogicalExpression",
      left: null,
      operator: deserializeLogicalOperator(pos + 12),
      right: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeConditionalExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ConditionalExpression",
      test: null,
      consequent: null,
      alternate: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeExpression(pos + 32);
  node.alternate = deserializeExpression(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeAssignmentExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "AssignmentExpression",
      operator: deserializeAssignmentOperator(pos + 12),
      left: null,
      right: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeAssignmentTarget(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ArrayPattern",
      decorators: [],
      elements: null,
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  parent = previousParent;
  return node;
}

function deserializeObjectAssignmentTarget(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ObjectPattern",
      decorators: [],
      properties: null,
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    properties = deserializeVecAssignmentTargetProperty(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetRest(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "RestElement",
      decorators: [],
      argument: null,
      optional: false,
      typeAnnotation: null,
      value: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeAssignmentTarget(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "AssignmentPattern",
      decorators: [],
      left: null,
      right: null,
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeAssignmentTarget(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
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
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: true,
      computed: false,
      optional: false,
      start,
      end,
      range: [start, end],
      parent,
    }),
    key = deserializeIdentifierReference(pos + 16),
    keyStart,
    keyEnd,
    value = {
      type: "Identifier",
      decorators: [],
      name: key.name,
      optional: false,
      typeAnnotation: null,
      start: (keyStart = key.start),
      end: (keyEnd = key.end),
      range: [keyStart, keyEnd],
      parent,
    },
    init = deserializeOptionExpression(pos + 48);
  if (init !== null) {
    let left = value;
    value = {
      type: "AssignmentPattern",
      decorators: [],
      left,
      right: init,
      optional: false,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    };
    left.parent = value;
    init.parent = value;
  }
  node.key = key;
  node.value = value;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: false,
      computed: deserializeBool(pos + 12),
      optional: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeAssignmentTargetMaybeDefault(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSequenceExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "SequenceExpression",
      expressions: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expressions = deserializeVecExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeSuper(pos) {
  let start, end;
  return {
    type: "Super",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeAwaitExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "AwaitExpression",
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeChainExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ChainExpression",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeChainElement(pos + 16);
  parent = previousParent;
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
  {
    let start,
      end,
      previousParent = parent;
    node = parent = {
      type: "ParenthesizedExpression",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    };
    node.expression = deserializeExpression(pos + 16);
    parent = previousParent;
  }
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ExpressionStatement",
      expression: null,
      directive: deserializeStr(pos + 64),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeStringLiteral(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeHashbang(pos) {
  let start, end;
  return {
    type: "Hashbang",
    value: deserializeStr(pos + 16),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeBlockStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "BlockStatement",
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.body = deserializeVecStatement(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "VariableDeclaration",
      kind: deserializeVariableDeclarationKind(pos + 12),
      declarations: null,
      declare: deserializeBool(pos + 13),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.declarations = deserializeVecVariableDeclarator(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "VariableDeclarator",
      id: null,
      init: null,
      definite: deserializeBool(pos + 13),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    pattern = deserializeBindingPattern(pos + 16);
  {
    let previousParent = parent;
    parent = pattern;
    let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
    if (typeAnnotation !== null) {
      pattern.typeAnnotation = typeAnnotation;
      pattern.range[1] = pattern.end = typeAnnotation.end;
    }
    parent = previousParent;
  }
  node.id = pattern;
  node.init = deserializeOptionExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeEmptyStatement(pos) {
  let start, end;
  return {
    type: "EmptyStatement",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeExpressionStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ExpressionStatement",
      expression: null,
      directive: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeIfStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "IfStatement",
      test: null,
      consequent: null,
      alternate: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeStatement(pos + 32);
  node.alternate = deserializeOptionStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeDoWhileStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "DoWhileStatement",
      body: null,
      test: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.body = deserializeStatement(pos + 16);
  node.test = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeWhileStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "WhileStatement",
      test: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.test = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeForStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ForStatement",
      init: null,
      test: null,
      update: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.init = deserializeOptionForStatementInit(pos + 16);
  node.test = deserializeOptionExpression(pos + 32);
  node.update = deserializeOptionExpression(pos + 48);
  node.body = deserializeStatement(pos + 64);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ForInStatement",
      left: null,
      right: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ForOfStatement",
      await: deserializeBool(pos + 64),
      left: null,
      right: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeContinueStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ContinueStatement",
      label: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBreakStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "BreakStatement",
      label: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeReturnStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ReturnStatement",
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeOptionExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeWithStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "WithStatement",
      object: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSwitchStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "SwitchStatement",
      discriminant: null,
      cases: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.discriminant = deserializeExpression(pos + 16);
  node.cases = deserializeVecSwitchCase(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSwitchCase(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "SwitchCase",
      test: null,
      consequent: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.test = deserializeOptionExpression(pos + 16);
  node.consequent = deserializeVecStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeLabeledStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "LabeledStatement",
      label: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.label = deserializeLabelIdentifier(pos + 16);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeThrowStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ThrowStatement",
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTryStatement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TryStatement",
      block: null,
      handler: null,
      finalizer: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.block = deserializeBoxBlockStatement(pos + 16);
  node.handler = deserializeOptionBoxCatchClause(pos + 24);
  node.finalizer = deserializeOptionBoxBlockStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeCatchClause(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "CatchClause",
      param: null,
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.param = deserializeOptionCatchParameter(pos + 16);
  node.body = deserializeBoxBlockStatement(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeCatchParameter(pos) {
  let previousParent = parent,
    pattern = deserializeBindingPattern(pos + 16);
  {
    parent = pattern;
    let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
    pattern.typeAnnotation = typeAnnotation;
    if (typeAnnotation !== null) {
      pattern.end = typeAnnotation.end;
      pattern.range[1] = typeAnnotation.end;
    }
    parent = previousParent;
  }
  return pattern;
}

function deserializeDebuggerStatement(pos) {
  let start, end;
  return {
    type: "DebuggerStatement",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeBindingPattern(pos) {
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
      throw Error(`Unexpected discriminant ${uint8[pos]} for BindingPattern`);
  }
}

function deserializeAssignmentPattern(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "AssignmentPattern",
      decorators: [],
      left: null,
      right: null,
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeBindingPattern(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeObjectPattern(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ObjectPattern",
      decorators: [],
      properties: null,
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    properties = deserializeVecBindingProperty(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  parent = previousParent;
  return node;
}

function deserializeBindingProperty(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: deserializeBool(pos + 12),
      computed: deserializeBool(pos + 13),
      optional: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeBindingPattern(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeArrayPattern(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ArrayPattern",
      decorators: [],
      elements: null,
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    elements = deserializeVecOptionBindingPattern(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  parent = previousParent;
  return node;
}

function deserializeBindingRestElement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "RestElement",
      decorators: [],
      argument: null,
      optional: false,
      typeAnnotation: null,
      value: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeBindingPattern(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeFunction(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: deserializeFunctionType(pos + 88),
      id: null,
      generator: deserializeBool(pos + 89),
      async: deserializeBool(pos + 90),
      declare: deserializeBool(pos + 91),
      typeParameters: null,
      params: null,
      returnType: null,
      body: null,
      expression: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 64);
  {
    let thisParam = deserializeOptionBoxTSThisParameter(pos + 56);
    thisParam !== null && params.unshift(thisParam);
  }
  node.id = deserializeOptionBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 72);
  node.body = deserializeOptionBoxFunctionBody(pos + 80);
  parent = previousParent;
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
  let params = deserializeVecFormalParameter(pos + 16),
    restFieldPos32 = (pos >> 2) + 10;
  if (int32[restFieldPos32] !== 0 && int32[restFieldPos32 + 1] !== 0) {
    pos = int32[restFieldPos32];
    let start,
      end,
      previousParent = parent,
      rest = (parent = {
        type: "RestElement",
        decorators: [],
        argument: null,
        optional: false,
        typeAnnotation: null,
        value: null,
        start: (start = deserializeI32(pos + 40)),
        end: (end = deserializeI32(pos + 44)),
        range: [start, end],
        parent: previousParent,
      });
    rest.argument = deserializeBindingPattern(pos + 56);
    rest.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 72);
    if (rest.typeAnnotation !== null) {
      end = rest.typeAnnotation.end;
      rest.end = end;
      rest.range[1] = end;
    }
    params.push(rest);
    parent = previousParent;
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param,
    previousParent = parent,
    initializerFieldPos32 = (pos >> 2) + 16,
    hasInitializer = int32[initializerFieldPos32] !== 0 && int32[initializerFieldPos32 + 1] !== 0;
  {
    let accessibility = deserializeOptionTSAccessibility(pos + 13),
      readonly = deserializeBool(pos + 14),
      override = deserializeBool(pos + 15);
    if (accessibility === null && !readonly && !override) {
      let optional = deserializeBool(pos + 12);
      if (hasInitializer) {
        let start, end;
        param = parent = {
          type: "AssignmentPattern",
          decorators: null,
          left: null,
          right: null,
          optional,
          typeAnnotation: null,
          start: (start = deserializeI32(pos)),
          end: (end = deserializeI32(pos + 4)),
          range: [start, end],
          parent: previousParent,
        };
        param.decorators = deserializeVecDecorator(pos + 16);
        param.left = deserializeBindingPattern(pos + 40);
        param.left.decorators = [];
        param.left.optional = false;
        parent = param.left;
        let leftTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.left.typeAnnotation = leftTypeAnnotation;
        if (leftTypeAnnotation !== null) {
          param.left.end = leftTypeAnnotation.end;
          param.left.range[1] = leftTypeAnnotation.end;
        }
        parent = param;
        param.right = deserializeOptionBoxExpression(pos + 64);
      } else {
        param = deserializeBindingPattern(pos + 40);
        param.parent = previousParent;
        parent = param;
        param.decorators = deserializeVecDecorator(pos + 16);
        param.optional = optional;
        let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.typeAnnotation = typeAnnotation;
        if (typeAnnotation !== null) {
          param.end = typeAnnotation.end;
          param.range[1] = typeAnnotation.end;
        } else if (optional) {
          param.end = deserializeI32(pos + 4);
          param.range[1] = deserializeI32(pos + 4);
        }
        parent = previousParent;
      }
    } else {
      let start, end;
      param = parent = {
        type: "TSParameterProperty",
        accessibility,
        decorators: null,
        override,
        parameter: null,
        readonly,
        static: false,
        start: (start = deserializeI32(pos)),
        end: (end = deserializeI32(pos + 4)),
        range: [start, end],
        parent: previousParent,
      };
      param.decorators = deserializeVecDecorator(pos + 16);
      if (hasInitializer) {
        let pattern = deserializeBindingPattern(pos + 40),
          initializer = deserializeOptionBoxExpression(pos + 64),
          assignStart,
          assignEnd,
          assignParam = (parent = {
            type: "AssignmentPattern",
            decorators: [],
            left: null,
            right: null,
            optional: false,
            typeAnnotation: null,
            start: (assignStart = pattern.start),
            end: (assignEnd = initializer.end),
            range: [assignStart, assignEnd],
            parent: param,
          });
        assignParam.left = pattern;
        pattern.parent = assignParam;
        pattern.decorators = [];
        pattern.optional = false;
        parent = pattern;
        let patternTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        pattern.typeAnnotation = patternTypeAnnotation;
        if (patternTypeAnnotation !== null) {
          pattern.end = patternTypeAnnotation.end;
          pattern.range[1] = patternTypeAnnotation.end;
        }
        parent = assignParam;
        assignParam.right = initializer;
        initializer !== null && (initializer.parent = assignParam);
        param.parameter = assignParam;
      } else {
        param.parameter = deserializeBindingPattern(pos + 40);
        param.parameter.decorators = [];
        let paramOptional = deserializeBool(pos + 12);
        param.parameter.optional = paramOptional;
        parent = param.parameter;
        let paramTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.parameter.typeAnnotation = paramTypeAnnotation;
        if (paramTypeAnnotation !== null) {
          param.parameter.end = paramTypeAnnotation.end;
          param.parameter.range[1] = paramTypeAnnotation.end;
        } else if (paramOptional) {
          let paramEnd = deserializeI32(pos + 4);
          param.parameter.end = paramEnd;
          param.parameter.range[1] = paramEnd;
        }
        parent = param;
      }
    }
  }
  parent = previousParent;
  return param;
}

function deserializeFunctionBody(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "BlockStatement",
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    body = deserializeVecDirective(pos + 16);
  body.push(...deserializeVecStatement(pos + 40));
  node.body = body;
  parent = previousParent;
  return node;
}

function deserializeArrowFunctionExpression(pos) {
  let expression = deserializeBool(pos + 48),
    start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ArrowFunctionExpression",
      expression,
      async: deserializeBool(pos + 49),
      typeParameters: null,
      params: null,
      returnType: null,
      body: null,
      id: null,
      generator: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    body = deserializeBoxFunctionBody(pos + 40);
  if (expression === true) {
    body = body.body[0].expression;
    body.parent = parent;
  }
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeBoxFormalParameters(pos + 24);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  node.body = body;
  parent = previousParent;
  return node;
}

function deserializeYieldExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "YieldExpression",
      delegate: deserializeBool(pos + 12),
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeOptionExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeClass(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: deserializeClassType(pos + 136),
      decorators: null,
      id: null,
      typeParameters: null,
      superClass: null,
      superTypeArguments: null,
      implements: null,
      body: null,
      abstract: deserializeBool(pos + 137),
      declare: deserializeBool(pos + 138),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.id = deserializeOptionBindingIdentifier(pos + 40);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 72);
  node.superClass = deserializeOptionExpression(pos + 80);
  node.superTypeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 96);
  node.implements = deserializeVecTSClassImplements(pos + 104);
  node.body = deserializeBoxClassBody(pos + 128);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ClassBody",
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.body = deserializeVecClassElement(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: deserializeMethodDefinitionType(pos + 12),
      decorators: null,
      key: null,
      value: null,
      kind: deserializeMethodDefinitionKind(pos + 13),
      computed: deserializeBool(pos + 14),
      static: deserializeBool(pos + 15),
      override: deserializeBool(pos + 64),
      optional: deserializeBool(pos + 65),
      accessibility: deserializeOptionTSAccessibility(pos + 66),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.value = deserializeBoxFunction(pos + 56);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: deserializePropertyDefinitionType(pos + 12),
      decorators: null,
      key: null,
      typeAnnotation: null,
      value: null,
      computed: deserializeBool(pos + 13),
      static: deserializeBool(pos + 14),
      declare: deserializeBool(pos + 15),
      override: deserializeBool(pos + 80),
      optional: deserializeBool(pos + 81),
      definite: deserializeBool(pos + 82),
      readonly: deserializeBool(pos + 83),
      accessibility: deserializeOptionTSAccessibility(pos + 84),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  node.value = deserializeOptionExpression(pos + 64);
  parent = previousParent;
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
  let start, end;
  return {
    type: "PrivateIdentifier",
    name: deserializeStr(pos + 16),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeStaticBlock(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "StaticBlock",
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.body = deserializeVecStatement(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: deserializeAccessorPropertyType(pos + 12),
      decorators: null,
      key: null,
      typeAnnotation: null,
      value: null,
      computed: deserializeBool(pos + 13),
      static: deserializeBool(pos + 14),
      override: deserializeBool(pos + 15),
      definite: deserializeBool(pos + 80),
      accessibility: deserializeOptionTSAccessibility(pos + 81),
      declare: false,
      optional: false,
      readonly: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  node.value = deserializeOptionExpression(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeImportExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ImportExpression",
      source: null,
      options: null,
      phase: deserializeOptionImportPhase(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.source = deserializeExpression(pos + 16);
  node.options = deserializeOptionExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeImportDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ImportDeclaration",
      specifiers: null,
      source: null,
      phase: deserializeOptionImportPhase(pos + 12),
      attributes: null,
      importKind: deserializeImportOrExportKind(pos + 13),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 16);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 88);
  node.specifiers = specifiers;
  node.source = deserializeStringLiteral(pos + 40);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ImportSpecifier",
      imported: null,
      local: null,
      importKind: deserializeImportOrExportKind(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.imported = deserializeModuleExportName(pos + 16);
  node.local = deserializeBindingIdentifier(pos + 72);
  parent = previousParent;
  return node;
}

function deserializeImportDefaultSpecifier(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ImportDefaultSpecifier",
      local: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.local = deserializeBindingIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeImportNamespaceSpecifier(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ImportNamespaceSpecifier",
      local: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.local = deserializeBindingIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 16) };
}

function deserializeImportAttribute(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ImportAttribute",
      key: null,
      value: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.key = deserializeImportAttributeKey(pos + 16);
  node.value = deserializeStringLiteral(pos + 72);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      exportKind: deserializeImportOrExportKind(pos + 12),
      attributes: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    withClause = deserializeOptionBoxWithClause(pos + 104);
  node.declaration = deserializeOptionDeclaration(pos + 16);
  node.specifiers = deserializeVecExportSpecifier(pos + 32);
  node.source = deserializeOptionStringLiteral(pos + 56);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportDefaultDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ExportDefaultDeclaration",
      declaration: null,
      exportKind: "value",
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.declaration = deserializeExportDefaultDeclarationKind(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeExportAllDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ExportAllDeclaration",
      exported: null,
      source: null,
      attributes: null,
      exportKind: deserializeImportOrExportKind(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    withClause = deserializeOptionBoxWithClause(pos + 120);
  node.exported = deserializeOptionModuleExportName(pos + 16);
  node.source = deserializeStringLiteral(pos + 72);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportSpecifier(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "ExportSpecifier",
      local: null,
      exported: null,
      exportKind: deserializeImportOrExportKind(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.local = deserializeModuleExportName(pos + 16);
  node.exported = deserializeModuleExportName(pos + 72);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "V8IntrinsicExpression",
      name: null,
      arguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.name = deserializeIdentifierName(pos + 16);
  node.arguments = deserializeVecArgument(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeBooleanLiteral(pos) {
  let value = deserializeBool(pos + 12),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value,
      raw: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.raw = start === 0 && end === 0 ? null : value + "";
  parent = previousParent;
  return node;
}

function deserializeNullLiteral(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4);
  return {
    type: "Literal",
    value: null,
    raw: start === 0 && end === 0 ? null : "null",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeNumericLiteral(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4);
  return {
    type: "Literal",
    value: deserializeF64(pos + 32),
    raw:
      int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
        ? null
        : sourceText.slice(start, end),
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeStringLiteral(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw:
        int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
          ? null
          : sourceText.slice(start, end),
      start,
      end,
      range: [start, end],
      parent,
    }),
    value = deserializeStr(pos + 16);
  deserializeBool(pos + 12) &&
    (value = value.replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16))));
  node.value = value;
  parent = previousParent;
  return node;
}

function deserializeBigIntLiteral(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    bigint = deserializeStr(pos + 16);
  return {
    type: "Literal",
    value: BigInt(bigint),
    raw:
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end),
    bigint,
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeRegExpLiteral(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw:
        int32[(pos >> 2) + 12] === 0 && int32[(pos >> 2) + 13] === 0
          ? null
          : sourceText.slice(start, end),
      regex: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    regex = deserializeRegExp(pos + 16),
    value = null;
  try {
    value = new RegExp(regex.pattern, regex.flags);
  } catch {}
  node.value = value;
  node.regex = regex;
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXElement",
      openingElement: null,
      children: null,
      closingElement: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    closingElement = deserializeOptionBoxJSXClosingElement(pos + 48),
    openingElement = deserializeBoxJSXOpeningElement(pos + 16);
  closingElement === null && (openingElement.selfClosing = true);
  node.openingElement = openingElement;
  node.children = deserializeVecJSXChild(pos + 24);
  node.closingElement = closingElement;
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningElement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXOpeningElement",
      name: null,
      typeArguments: null,
      attributes: null,
      selfClosing: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.name = deserializeJSXElementName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.attributes = deserializeVecJSXAttributeItem(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeJSXClosingElement(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXClosingElement",
      name: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.name = deserializeJSXElementName(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSXFragment(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXFragment",
      openingFragment: null,
      children: null,
      closingFragment: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.openingFragment = deserializeJSXOpeningFragment(pos + 16);
  node.children = deserializeVecJSXChild(pos + 32);
  node.closingFragment = deserializeJSXClosingFragment(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningFragment(pos) {
  let start, end;
  return {
    type: "JSXOpeningFragment",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeJSXClosingFragment(pos) {
  let start, end;
  return {
    type: "JSXClosingFragment",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
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
        range: ident.range,
        parent,
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
        range: thisExpr.range,
        parent,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXNamespacedName",
      namespace: null,
      name: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.namespace = deserializeJSXIdentifier(pos + 16);
  node.name = deserializeJSXIdentifier(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeJSXMemberExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXMemberExpression",
      object: null,
      property: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.object = deserializeJSXMemberExpressionObject(pos + 16);
  node.property = deserializeJSXIdentifier(pos + 32);
  parent = previousParent;
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
        range: ident.range,
        parent,
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
        range: thisExpr.range,
        parent,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXExpressionContainer",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeJSXExpression(pos + 16);
  parent = previousParent;
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
      return deserializeBoxJSXEmptyExpression(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXExpression`);
  }
}

function deserializeJSXEmptyExpression(pos) {
  let start, end;
  return {
    type: "JSXEmptyExpression",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXAttribute",
      name: null,
      value: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.name = deserializeJSXAttributeName(pos + 16);
  node.value = deserializeOptionJSXAttributeValue(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeJSXSpreadAttribute(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXSpreadAttribute",
      argument: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
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
  let start, end;
  return {
    type: "JSXIdentifier",
    name: deserializeStr(pos + 16),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "JSXSpreadChild",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSXText(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4);
  return {
    type: "JSXText",
    value: deserializeStr(pos + 16),
    raw:
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end),
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSThisParameter(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: [],
      name: "this",
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSEnumDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSEnumDeclaration",
      id: null,
      body: null,
      const: deserializeBool(pos + 12),
      declare: deserializeBool(pos + 13),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.body = deserializeTSEnumBody(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSEnumBody(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSEnumBody",
      members: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.members = deserializeVecTSEnumMember(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSEnumMember(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSEnumMember",
      id: null,
      initializer: null,
      computed: deserializeU8(pos + 16) > 1,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.id = deserializeTSEnumMemberName(pos + 16);
  node.initializer = deserializeOptionExpression(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeAnnotation",
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSLiteralType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSLiteralType",
      literal: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.literal = deserializeTSLiteral(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSConditionalType",
      checkType: null,
      extendsType: null,
      trueType: null,
      falseType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.checkType = deserializeTSType(pos + 16);
  node.extendsType = deserializeTSType(pos + 32);
  node.trueType = deserializeTSType(pos + 48);
  node.falseType = deserializeTSType(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeTSUnionType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSUnionType",
      types: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.types = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSIntersectionType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSIntersectionType",
      types: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.types = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSParenthesizedType(pos) {
  let node;
  {
    let start,
      end,
      previousParent = parent;
    node = parent = {
      type: "TSParenthesizedType",
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    };
    node.typeAnnotation = deserializeTSType(pos + 16);
    parent = previousParent;
  }
  return node;
}

function deserializeTSTypeOperator(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeOperator",
      operator: deserializeTSTypeOperatorOperator(pos + 12),
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSArrayType",
      elementType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.elementType = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSIndexedAccessType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSIndexedAccessType",
      objectType: null,
      indexType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.objectType = deserializeTSType(pos + 16);
  node.indexType = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTupleType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTupleType",
      elementTypes: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.elementTypes = deserializeVecTSTupleElement(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNamedTupleMember(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSNamedTupleMember",
      label: null,
      elementType: null,
      optional: deserializeBool(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.label = deserializeIdentifierName(pos + 16);
  node.elementType = deserializeTSTupleElement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSOptionalType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSOptionalType",
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSRestType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSRestType",
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
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
  let start, end;
  return {
    type: "TSAnyKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSStringKeyword(pos) {
  let start, end;
  return {
    type: "TSStringKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSBooleanKeyword(pos) {
  let start, end;
  return {
    type: "TSBooleanKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSNumberKeyword(pos) {
  let start, end;
  return {
    type: "TSNumberKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSNeverKeyword(pos) {
  let start, end;
  return {
    type: "TSNeverKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  let start, end;
  return {
    type: "TSIntrinsicKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSUnknownKeyword(pos) {
  let start, end;
  return {
    type: "TSUnknownKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSNullKeyword(pos) {
  let start, end;
  return {
    type: "TSNullKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSUndefinedKeyword(pos) {
  let start, end;
  return {
    type: "TSUndefinedKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSVoidKeyword(pos) {
  let start, end;
  return {
    type: "TSVoidKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSSymbolKeyword(pos) {
  let start, end;
  return {
    type: "TSSymbolKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSThisType(pos) {
  let start, end;
  return {
    type: "TSThisType",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSObjectKeyword(pos) {
  let start, end;
  return {
    type: "TSObjectKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSBigIntKeyword(pos) {
  let start, end;
  return {
    type: "TSBigIntKeyword",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeTSTypeReference(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeReference",
      typeName: null,
      typeArguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeName = deserializeTSTypeName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeTSTypeName(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterInstantiation(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeParameterInstantiation",
      params: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.params = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameter(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeParameter",
      name: null,
      constraint: null,
      default: null,
      in: deserializeBool(pos + 12),
      out: deserializeBool(pos + 13),
      const: deserializeBool(pos + 14),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.name = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeOptionTSType(pos + 48);
  node.default = deserializeOptionTSType(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeParameterDeclaration",
      params: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.params = deserializeVecTSTypeParameter(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAliasDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeAliasDeclaration",
      id: null,
      typeParameters: null,
      typeAnnotation: null,
      declare: deserializeBool(pos + 72),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.typeAnnotation = deserializeTSType(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeTSAccessibility(pos) {
  switch (uint8[pos]) {
    case 0:
      return "private";
    case 1:
      return "protected";
    case 2:
      return "public";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSAccessibility`);
  }
}

function deserializeTSClassImplements(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSClassImplements",
      expression: null,
      typeArguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    expression = deserializeTSTypeName(pos + 16);
  if (expression.type === "TSQualifiedName") {
    let object = expression.left,
      { right } = expression,
      start,
      end,
      previous = (expression = {
        type: "MemberExpression",
        object,
        property: right,
        optional: false,
        computed: false,
        start: (start = expression.start),
        end: (end = expression.end),
        range: [start, end],
        parent,
      });
    right.parent = previous;
    for (;;) {
      if (object.type !== "TSQualifiedName") {
        object.parent = previous;
        break;
      }
      let { left, right } = object;
      previous = previous.object = {
        type: "MemberExpression",
        object: left,
        property: right,
        optional: false,
        computed: false,
        start: (start = object.start),
        end: (end = object.end),
        range: [start, end],
        parent: previous,
      };
      right.parent = previous;
      object = left;
    }
  }
  node.expression = expression;
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSInterfaceDeclaration",
      id: null,
      typeParameters: null,
      extends: null,
      body: null,
      declare: deserializeBool(pos + 88),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.extends = deserializeVecTSInterfaceHeritage(pos + 56);
  node.body = deserializeBoxTSInterfaceBody(pos + 80);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceBody(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSInterfaceBody",
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.body = deserializeVecTSSignature(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSPropertySignature(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSPropertySignature",
      computed: deserializeBool(pos + 12),
      optional: deserializeBool(pos + 13),
      readonly: deserializeBool(pos + 14),
      key: null,
      typeAnnotation: null,
      accessibility: null,
      static: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSIndexSignature",
      parameters: null,
      typeAnnotation: null,
      readonly: deserializeBool(pos + 12),
      static: deserializeBool(pos + 13),
      accessibility: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.parameters = deserializeVecTSIndexSignatureName(pos + 16);
  node.typeAnnotation = deserializeBoxTSTypeAnnotation(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTSCallSignatureDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSCallSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 32),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 40);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSMethodSignature",
      key: null,
      computed: deserializeBool(pos + 64),
      optional: deserializeBool(pos + 65),
      kind: deserializeTSMethodSignatureKind(pos + 66),
      typeParameters: null,
      params: null,
      returnType: null,
      accessibility: null,
      readonly: false,
      static: false,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 48),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 40);
  thisParam !== null && params.unshift(thisParam);
  node.key = deserializePropertyKey(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 32);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeTSConstructSignatureDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSConstructSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeBoxFormalParameters(pos + 24);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSIndexSignatureName(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: [],
      name: deserializeStr(pos + 16),
      optional: false,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceHeritage(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSInterfaceHeritage",
      expression: null,
      typeArguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypePredicate(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypePredicate",
      parameterName: null,
      asserts: deserializeBool(pos + 12),
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.parameterName = deserializeTSTypePredicateName(pos + 16);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypePredicateName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxIdentifierName(pos + 8);
    case 1:
      return deserializeBoxTSThisType(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypePredicateName`);
  }
}

function deserializeTSModuleDeclaration(pos) {
  let kind = deserializeTSModuleDeclarationKind(pos + 88),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    declare = deserializeBool(pos + 89),
    node,
    previousParent = parent,
    body = deserializeOptionTSModuleDeclarationBody(pos + 72);
  if (body === null) {
    node = parent = {
      type: "TSModuleDeclaration",
      id: null,
      // No `body` field
      kind,
      declare,
      global: false,
      start,
      end,
      range: [start, end],
      parent,
    };
    node.id = deserializeTSModuleDeclarationName(pos + 16);
  } else {
    node = parent = {
      type: "TSModuleDeclaration",
      id: null,
      body,
      kind,
      declare,
      global: false,
      start,
      end,
      range: [start, end],
      parent,
    };
    let id = deserializeTSModuleDeclarationName(pos + 16);
    if (body.type === "TSModuleBlock") {
      node.id = id;
      body.parent = node;
    } else {
      let innerId = body.id;
      if (innerId.type === "Identifier") {
        let start,
          end,
          outerId =
            (node.id =
            parent =
              {
                type: "TSQualifiedName",
                left: id,
                right: innerId,
                start: (start = id.start),
                end: (end = innerId.end),
                range: [start, end],
                parent: node,
              });
        id.parent = innerId.parent = outerId;
      } else {
        // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with `id` of
        // this module on left, and previous `left` of innermost `TSQualifiedName` on right
        node.id = innerId;
        innerId.parent = node;
        let { start } = id;
        for (;;) {
          innerId.start = innerId.range[0] = start;
          if (innerId.left.type === "Identifier") break;
          innerId = innerId.left;
        }
        let end,
          right = innerId.left;
        id.parent =
          right.parent =
          innerId.left =
            {
              type: "TSQualifiedName",
              left: id,
              right,
              start,
              end: (end = right.end),
              range: [start, end],
              parent: innerId,
            };
      }
      if (Object.hasOwn(body, "body")) {
        body = body.body;
        node.body = body;
        body.parent = node;
      } else body = null;
    }
  }
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSModuleDeclaration",
      id: null,
      body: null,
      kind: "global",
      declare: deserializeBool(pos + 88),
      global: true,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    keywordStart,
    keywordEnd;
  node.id = {
    type: "Identifier",
    decorators: [],
    name: "global",
    optional: false,
    typeAnnotation: null,
    start: (keywordStart = deserializeI32(pos + 16)),
    end: (keywordEnd = deserializeI32(pos + 20)),
    range: [keywordStart, keywordEnd],
    parent,
  };
  node.body = deserializeTSModuleBlock(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSModuleBlock(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSModuleBlock",
      body: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    body = deserializeVecDirective(pos + 16);
  body.push(...deserializeVecStatement(pos + 40));
  node.body = body;
  parent = previousParent;
  return node;
}

function deserializeTSTypeLiteral(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeLiteral",
      members: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.members = deserializeVecTSSignature(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSInferType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSInferType",
      typeParameter: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeParameter = deserializeBoxTSTypeParameter(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeQuery(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeQuery",
      exprName: null,
      typeArguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.exprName = deserializeTSTypeQueryExprName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSImportType",
      source: null,
      options: null,
      qualifier: null,
      typeArguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.source = deserializeStringLiteral(pos + 16);
  node.options = deserializeOptionBoxObjectExpression(pos + 64);
  node.qualifier = deserializeOptionTSImportTypeQualifier(pos + 72);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 88);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.left = deserializeTSImportTypeQualifier(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSFunctionType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSFunctionType",
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 32),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = params;
  node.returnType = deserializeBoxTSTypeAnnotation(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTSConstructorType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSConstructorType",
      abstract: deserializeBool(pos + 40),
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeBoxFormalParameters(pos + 24);
  node.returnType = deserializeBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSMappedType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSMappedType",
      key: null,
      constraint: null,
      nameType: null,
      typeAnnotation: null,
      optional: null,
      readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 97),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    }),
    optional = deserializeOptionTSMappedTypeModifierOperator(pos + 96);
  optional === null && (optional = false);
  node.key = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeTSType(pos + 48);
  node.nameType = deserializeOptionTSType(pos + 64);
  node.typeAnnotation = deserializeOptionTSType(pos + 80);
  node.optional = optional;
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTemplateLiteralType",
      quasis: null,
      types: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.types = deserializeVecTSType(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTSAsExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSAsExpression",
      expression: null,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSSatisfiesExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSSatisfiesExpression",
      expression: null,
      typeAnnotation: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAssertion(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSTypeAssertion",
      typeAnnotation: null,
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  node.expression = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSImportEqualsDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSImportEqualsDeclaration",
      id: null,
      moduleReference: null,
      importKind: deserializeImportOrExportKind(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.moduleReference = deserializeTSModuleReference(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSModuleReference(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBoxTSExternalModuleReference(pos + 8);
    case 1:
      return deserializeBoxIdentifierReference(pos + 8);
    case 2:
      return deserializeBoxTSQualifiedName(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleReference`);
  }
}

function deserializeTSExternalModuleReference(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSExternalModuleReference",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeStringLiteral(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNonNullExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSNonNullExpression",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeDecorator(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "Decorator",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSExportAssignment(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSExportAssignment",
      expression: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSNamespaceExportDeclaration",
      id: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.id = deserializeIdentifierName(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSInstantiationExpression(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSInstantiationExpression",
      expression: null,
      typeArguments: null,
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeArguments = deserializeBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
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
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSJSDocNullableType",
      typeAnnotation: null,
      postfix: deserializeBool(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSDocNonNullableType(pos) {
  let start,
    end,
    previousParent = parent,
    node = (parent = {
      type: "TSJSDocNonNullableType",
      typeAnnotation: null,
      postfix: deserializeBool(pos + 12),
      start: (start = deserializeI32(pos)),
      end: (end = deserializeI32(pos + 4)),
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSDocUnknownType(pos) {
  let start, end;
  return {
    type: "TSJSDocUnknownType",
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
    parent,
  };
}

function deserializeCommentKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "Line";
    case 1:
      return "Block";
    case 2:
      return "Block";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for CommentKind`);
  }
}

function deserializeComment(pos) {
  let type = deserializeCommentKind(pos + 12),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4);
  return {
    type,
    value: sourceText.slice(start + 2, end - (type === "Line" ? 0 : 2)),
    start,
    end,
    range: [start, end],
  };
}

function deserializeModuleKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "script";
    case 1:
      return "module";
    case 3:
      return "commonjs";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ModuleKind`);
  }
}

function deserializeSpan(pos) {
  return {
    start: deserializeI32(pos),
    end: deserializeI32(pos + 4),
  };
}

function deserializeNameSpan(pos) {
  let start, end;
  return {
    value: deserializeStr(pos + 8),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
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
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      return {
        kind: "NamespaceObject",
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    case 2:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: "Default",
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
  let start, end;
  return {
    moduleRequest: deserializeOptionNameSpan(pos + 16),
    importName: deserializeExportImportName(pos + 40),
    exportName: deserializeExportExportName(pos + 72),
    localName: deserializeExportLocalName(pos + 104),
    isType: deserializeBool(pos + 136),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
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
        range: nameSpan.range,
      };
    case 1:
      return {
        kind: "All",
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    case 2:
      return {
        kind: "AllButDefault",
        name: null,
        start: null,
        end: null,
        range: [null, null],
      };
    case 3:
      return {
        kind: "None",
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
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: "Default",
        name: null,
        start,
        end,
        range: [start, end],
      };
    case 2:
      return {
        kind: "None",
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
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 1:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Default",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
        range: nameSpan.range,
      };
    case 2:
      return {
        kind: "None",
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
  let start, end;
  return {
    moduleRequest: deserializeSpan(pos + 8),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
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

function deserializeRawTransferData(pos) {
  return {
    program: deserializeProgram(pos),
    comments: deserializeVecComment(pos + 144),
    module: deserializeEcmaScriptModule(pos + 168),
    errors: deserializeVecError(pos + 272),
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
  let start, end;
  return {
    message: deserializeOptionStr(pos + 8),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
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
  let start, end;
  return {
    moduleRequest: deserializeNameSpan(pos + 8),
    entries: deserializeVecImportEntry(pos + 32),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
  };
}

function deserializeStaticExport(pos) {
  let start, end;
  return {
    entries: deserializeVecExportEntry(pos + 8),
    start: (start = deserializeI32(pos)),
    end: (end = deserializeI32(pos + 4)),
    range: [start, end],
  };
}

function deserializeStr(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return "";
  pos = int32[pos32];
  let end = pos + len;
  if (end <= firstNonAsciiPos) return sourceTextLatin.substr(pos, len);
  // Use `utf8Slice` for strings longer than 64 bytes
  if (len > 64) return utf8Slice.call(uint8, pos, end);
  if (pos < sourceEndPos) {
    // Check if all bytes are ASCII, use `utf8Slice` if not
    for (let i = pos; i < end; i++) if (uint8[i] >= 128) return utf8Slice.call(uint8, pos, end);
    // String is all ASCII, so slice from `sourceTextLatin`
    return sourceTextLatin.substr(pos, len);
  }
  // String is not in source region - use `fromCharCode.apply` with a temp array of correct length.
  // Copy bytes into temp array.
  // If any byte is non-ASCII, use `utf8Slice`.
  let arr = stringDecodeArrays[len];
  for (let i = 0; i < len; i++) {
    let b = uint8[pos + i];
    if (b >= 128) return utf8Slice.call(uint8, pos, end);
    arr[i] = b;
  }
  // Call `fromCharCode` with temp array
  return fromCharCode.apply(null, arr);
}

function deserializeVecComment(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeComment(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionHashbang(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeHashbang(pos);
}

function deserializeVecDirective(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 80;
  for (; pos !== endPos; ) {
    arr.push(deserializeDirective(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecStatement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeStatement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxBooleanLiteral(pos) {
  return deserializeBooleanLiteral(int32[pos >> 2]);
}

function deserializeBoxNullLiteral(pos) {
  return deserializeNullLiteral(int32[pos >> 2]);
}

function deserializeBoxNumericLiteral(pos) {
  return deserializeNumericLiteral(int32[pos >> 2]);
}

function deserializeBoxBigIntLiteral(pos) {
  return deserializeBigIntLiteral(int32[pos >> 2]);
}

function deserializeBoxRegExpLiteral(pos) {
  return deserializeRegExpLiteral(int32[pos >> 2]);
}

function deserializeBoxStringLiteral(pos) {
  return deserializeStringLiteral(int32[pos >> 2]);
}

function deserializeBoxTemplateLiteral(pos) {
  return deserializeTemplateLiteral(int32[pos >> 2]);
}

function deserializeBoxIdentifierReference(pos) {
  return deserializeIdentifierReference(int32[pos >> 2]);
}

function deserializeBoxMetaProperty(pos) {
  return deserializeMetaProperty(int32[pos >> 2]);
}

function deserializeBoxSuper(pos) {
  return deserializeSuper(int32[pos >> 2]);
}

function deserializeBoxArrayExpression(pos) {
  return deserializeArrayExpression(int32[pos >> 2]);
}

function deserializeBoxArrowFunctionExpression(pos) {
  return deserializeArrowFunctionExpression(int32[pos >> 2]);
}

function deserializeBoxAssignmentExpression(pos) {
  return deserializeAssignmentExpression(int32[pos >> 2]);
}

function deserializeBoxAwaitExpression(pos) {
  return deserializeAwaitExpression(int32[pos >> 2]);
}

function deserializeBoxBinaryExpression(pos) {
  return deserializeBinaryExpression(int32[pos >> 2]);
}

function deserializeBoxCallExpression(pos) {
  return deserializeCallExpression(int32[pos >> 2]);
}

function deserializeBoxChainExpression(pos) {
  return deserializeChainExpression(int32[pos >> 2]);
}

function deserializeBoxClass(pos) {
  return deserializeClass(int32[pos >> 2]);
}

function deserializeBoxConditionalExpression(pos) {
  return deserializeConditionalExpression(int32[pos >> 2]);
}

function deserializeBoxFunction(pos) {
  return deserializeFunction(int32[pos >> 2]);
}

function deserializeBoxImportExpression(pos) {
  return deserializeImportExpression(int32[pos >> 2]);
}

function deserializeBoxLogicalExpression(pos) {
  return deserializeLogicalExpression(int32[pos >> 2]);
}

function deserializeBoxNewExpression(pos) {
  return deserializeNewExpression(int32[pos >> 2]);
}

function deserializeBoxObjectExpression(pos) {
  return deserializeObjectExpression(int32[pos >> 2]);
}

function deserializeBoxParenthesizedExpression(pos) {
  return deserializeParenthesizedExpression(int32[pos >> 2]);
}

function deserializeBoxSequenceExpression(pos) {
  return deserializeSequenceExpression(int32[pos >> 2]);
}

function deserializeBoxTaggedTemplateExpression(pos) {
  return deserializeTaggedTemplateExpression(int32[pos >> 2]);
}

function deserializeBoxThisExpression(pos) {
  return deserializeThisExpression(int32[pos >> 2]);
}

function deserializeBoxUnaryExpression(pos) {
  return deserializeUnaryExpression(int32[pos >> 2]);
}

function deserializeBoxUpdateExpression(pos) {
  return deserializeUpdateExpression(int32[pos >> 2]);
}

function deserializeBoxYieldExpression(pos) {
  return deserializeYieldExpression(int32[pos >> 2]);
}

function deserializeBoxPrivateInExpression(pos) {
  return deserializePrivateInExpression(int32[pos >> 2]);
}

function deserializeBoxJSXElement(pos) {
  return deserializeJSXElement(int32[pos >> 2]);
}

function deserializeBoxJSXFragment(pos) {
  return deserializeJSXFragment(int32[pos >> 2]);
}

function deserializeBoxTSAsExpression(pos) {
  return deserializeTSAsExpression(int32[pos >> 2]);
}

function deserializeBoxTSSatisfiesExpression(pos) {
  return deserializeTSSatisfiesExpression(int32[pos >> 2]);
}

function deserializeBoxTSTypeAssertion(pos) {
  return deserializeTSTypeAssertion(int32[pos >> 2]);
}

function deserializeBoxTSNonNullExpression(pos) {
  return deserializeTSNonNullExpression(int32[pos >> 2]);
}

function deserializeBoxTSInstantiationExpression(pos) {
  return deserializeTSInstantiationExpression(int32[pos >> 2]);
}

function deserializeBoxV8IntrinsicExpression(pos) {
  return deserializeV8IntrinsicExpression(int32[pos >> 2]);
}

function deserializeVecArrayExpressionElement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeArrayExpressionElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxSpreadElement(pos) {
  return deserializeSpreadElement(int32[pos >> 2]);
}

function deserializeBoxElision(pos) {
  return deserializeElision(int32[pos >> 2]);
}

function deserializeVecObjectPropertyKind(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeObjectPropertyKind(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxObjectProperty(pos) {
  return deserializeObjectProperty(int32[pos >> 2]);
}

function deserializeBool(pos) {
  return uint8[pos] === 1;
}

function deserializeBoxIdentifierName(pos) {
  return deserializeIdentifierName(int32[pos >> 2]);
}

function deserializeBoxPrivateIdentifier(pos) {
  return deserializePrivateIdentifier(int32[pos >> 2]);
}

function deserializeVecTemplateElement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 48;
  for (; pos !== endPos; ) {
    arr.push(deserializeTemplateElement(pos));
    pos += 48;
  }
  return arr;
}

function deserializeVecExpression(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeExpression(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSTypeParameterInstantiation(pos) {
  return deserializeTSTypeParameterInstantiation(int32[pos >> 2]);
}

function deserializeOptionBoxTSTypeParameterInstantiation(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxTSTypeParameterInstantiation(pos);
}

function deserializeOptionStr(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0 ? null : deserializeStr(pos);
}

function deserializeBoxComputedMemberExpression(pos) {
  return deserializeComputedMemberExpression(int32[pos >> 2]);
}

function deserializeBoxStaticMemberExpression(pos) {
  return deserializeStaticMemberExpression(int32[pos >> 2]);
}

function deserializeBoxPrivateFieldExpression(pos) {
  return deserializePrivateFieldExpression(int32[pos >> 2]);
}

function deserializeVecArgument(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeArgument(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxArrayAssignmentTarget(pos) {
  return deserializeArrayAssignmentTarget(int32[pos >> 2]);
}

function deserializeBoxObjectAssignmentTarget(pos) {
  return deserializeObjectAssignmentTarget(int32[pos >> 2]);
}

function deserializeOptionAssignmentTargetMaybeDefault(pos) {
  return uint8[pos] === 51 ? null : deserializeAssignmentTargetMaybeDefault(pos);
}

function deserializeVecOptionAssignmentTargetMaybeDefault(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeOptionAssignmentTargetMaybeDefault(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxAssignmentTargetRest(pos) {
  return deserializeAssignmentTargetRest(int32[pos >> 2]);
}

function deserializeOptionBoxAssignmentTargetRest(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxAssignmentTargetRest(pos);
}

function deserializeVecAssignmentTargetProperty(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeAssignmentTargetProperty(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxAssignmentTargetWithDefault(pos) {
  return deserializeAssignmentTargetWithDefault(int32[pos >> 2]);
}

function deserializeBoxAssignmentTargetPropertyIdentifier(pos) {
  return deserializeAssignmentTargetPropertyIdentifier(int32[pos >> 2]);
}

function deserializeBoxAssignmentTargetPropertyProperty(pos) {
  return deserializeAssignmentTargetPropertyProperty(int32[pos >> 2]);
}

function deserializeOptionExpression(pos) {
  return uint8[pos] === 51 ? null : deserializeExpression(pos);
}

function deserializeBoxBlockStatement(pos) {
  return deserializeBlockStatement(int32[pos >> 2]);
}

function deserializeBoxBreakStatement(pos) {
  return deserializeBreakStatement(int32[pos >> 2]);
}

function deserializeBoxContinueStatement(pos) {
  return deserializeContinueStatement(int32[pos >> 2]);
}

function deserializeBoxDebuggerStatement(pos) {
  return deserializeDebuggerStatement(int32[pos >> 2]);
}

function deserializeBoxDoWhileStatement(pos) {
  return deserializeDoWhileStatement(int32[pos >> 2]);
}

function deserializeBoxEmptyStatement(pos) {
  return deserializeEmptyStatement(int32[pos >> 2]);
}

function deserializeBoxExpressionStatement(pos) {
  return deserializeExpressionStatement(int32[pos >> 2]);
}

function deserializeBoxForInStatement(pos) {
  return deserializeForInStatement(int32[pos >> 2]);
}

function deserializeBoxForOfStatement(pos) {
  return deserializeForOfStatement(int32[pos >> 2]);
}

function deserializeBoxForStatement(pos) {
  return deserializeForStatement(int32[pos >> 2]);
}

function deserializeBoxIfStatement(pos) {
  return deserializeIfStatement(int32[pos >> 2]);
}

function deserializeBoxLabeledStatement(pos) {
  return deserializeLabeledStatement(int32[pos >> 2]);
}

function deserializeBoxReturnStatement(pos) {
  return deserializeReturnStatement(int32[pos >> 2]);
}

function deserializeBoxSwitchStatement(pos) {
  return deserializeSwitchStatement(int32[pos >> 2]);
}

function deserializeBoxThrowStatement(pos) {
  return deserializeThrowStatement(int32[pos >> 2]);
}

function deserializeBoxTryStatement(pos) {
  return deserializeTryStatement(int32[pos >> 2]);
}

function deserializeBoxWhileStatement(pos) {
  return deserializeWhileStatement(int32[pos >> 2]);
}

function deserializeBoxWithStatement(pos) {
  return deserializeWithStatement(int32[pos >> 2]);
}

function deserializeBoxVariableDeclaration(pos) {
  return deserializeVariableDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSTypeAliasDeclaration(pos) {
  return deserializeTSTypeAliasDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSInterfaceDeclaration(pos) {
  return deserializeTSInterfaceDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSEnumDeclaration(pos) {
  return deserializeTSEnumDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSModuleDeclaration(pos) {
  return deserializeTSModuleDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSGlobalDeclaration(pos) {
  return deserializeTSGlobalDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSImportEqualsDeclaration(pos) {
  return deserializeTSImportEqualsDeclaration(int32[pos >> 2]);
}

function deserializeVecVariableDeclarator(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 56;
  for (; pos !== endPos; ) {
    arr.push(deserializeVariableDeclarator(pos));
    pos += 56;
  }
  return arr;
}

function deserializeBoxTSTypeAnnotation(pos) {
  return deserializeTSTypeAnnotation(int32[pos >> 2]);
}

function deserializeOptionBoxTSTypeAnnotation(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxTSTypeAnnotation(pos);
}

function deserializeOptionStatement(pos) {
  return uint8[pos] === 70 ? null : deserializeStatement(pos);
}

function deserializeOptionForStatementInit(pos) {
  return uint8[pos] === 65 ? null : deserializeForStatementInit(pos);
}

function deserializeOptionLabelIdentifier(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeLabelIdentifier(pos);
}

function deserializeVecSwitchCase(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 56;
  for (; pos !== endPos; ) {
    arr.push(deserializeSwitchCase(pos));
    pos += 56;
  }
  return arr;
}

function deserializeBoxCatchClause(pos) {
  return deserializeCatchClause(int32[pos >> 2]);
}

function deserializeOptionBoxCatchClause(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxCatchClause(pos);
}

function deserializeOptionBoxBlockStatement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxBlockStatement(pos);
}

function deserializeOptionCatchParameter(pos) {
  return uint8[pos + 16] === 4 ? null : deserializeCatchParameter(pos);
}

function deserializeBoxBindingIdentifier(pos) {
  return deserializeBindingIdentifier(int32[pos >> 2]);
}

function deserializeBoxObjectPattern(pos) {
  return deserializeObjectPattern(int32[pos >> 2]);
}

function deserializeBoxArrayPattern(pos) {
  return deserializeArrayPattern(int32[pos >> 2]);
}

function deserializeBoxAssignmentPattern(pos) {
  return deserializeAssignmentPattern(int32[pos >> 2]);
}

function deserializeVecBindingProperty(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 48;
  for (; pos !== endPos; ) {
    arr.push(deserializeBindingProperty(pos));
    pos += 48;
  }
  return arr;
}

function deserializeBoxBindingRestElement(pos) {
  return deserializeBindingRestElement(int32[pos >> 2]);
}

function deserializeOptionBoxBindingRestElement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxBindingRestElement(pos);
}

function deserializeOptionBindingPattern(pos) {
  return uint8[pos] === 4 ? null : deserializeBindingPattern(pos);
}

function deserializeVecOptionBindingPattern(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeOptionBindingPattern(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionBindingIdentifier(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeBindingIdentifier(pos);
}

function deserializeBoxTSTypeParameterDeclaration(pos) {
  return deserializeTSTypeParameterDeclaration(int32[pos >> 2]);
}

function deserializeOptionBoxTSTypeParameterDeclaration(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxTSTypeParameterDeclaration(pos);
}

function deserializeBoxTSThisParameter(pos) {
  return deserializeTSThisParameter(int32[pos >> 2]);
}

function deserializeOptionBoxTSThisParameter(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxTSThisParameter(pos);
}

function deserializeBoxFormalParameters(pos) {
  return deserializeFormalParameters(int32[pos >> 2]);
}

function deserializeBoxFunctionBody(pos) {
  return deserializeFunctionBody(int32[pos >> 2]);
}

function deserializeOptionBoxFunctionBody(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxFunctionBody(pos);
}

function deserializeVecFormalParameter(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 72;
  for (; pos !== endPos; ) {
    arr.push(deserializeFormalParameter(pos));
    pos += 72;
  }
  return arr;
}

function deserializeVecDecorator(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 5);
  for (; pos !== endPos; ) {
    arr.push(deserializeDecorator(pos));
    pos += 32;
  }
  return arr;
}

function deserializeBoxExpression(pos) {
  return deserializeExpression(int32[pos >> 2]);
}

function deserializeOptionBoxExpression(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxExpression(pos);
}

function deserializeOptionTSAccessibility(pos) {
  return uint8[pos] === 3 ? null : deserializeTSAccessibility(pos);
}

function deserializeVecTSClassImplements(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 40;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSClassImplements(pos));
    pos += 40;
  }
  return arr;
}

function deserializeBoxClassBody(pos) {
  return deserializeClassBody(int32[pos >> 2]);
}

function deserializeVecClassElement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeClassElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxStaticBlock(pos) {
  return deserializeStaticBlock(int32[pos >> 2]);
}

function deserializeBoxMethodDefinition(pos) {
  return deserializeMethodDefinition(int32[pos >> 2]);
}

function deserializeBoxPropertyDefinition(pos) {
  return deserializePropertyDefinition(int32[pos >> 2]);
}

function deserializeBoxAccessorProperty(pos) {
  return deserializeAccessorProperty(int32[pos >> 2]);
}

function deserializeBoxTSIndexSignature(pos) {
  return deserializeTSIndexSignature(int32[pos >> 2]);
}

function deserializeBoxImportDeclaration(pos) {
  return deserializeImportDeclaration(int32[pos >> 2]);
}

function deserializeBoxExportAllDeclaration(pos) {
  return deserializeExportAllDeclaration(int32[pos >> 2]);
}

function deserializeBoxExportDefaultDeclaration(pos) {
  return deserializeExportDefaultDeclaration(int32[pos >> 2]);
}

function deserializeBoxExportNamedDeclaration(pos) {
  return deserializeExportNamedDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSExportAssignment(pos) {
  return deserializeTSExportAssignment(int32[pos >> 2]);
}

function deserializeBoxTSNamespaceExportDeclaration(pos) {
  return deserializeTSNamespaceExportDeclaration(int32[pos >> 2]);
}

function deserializeOptionImportPhase(pos) {
  return uint8[pos] === 2 ? null : deserializeImportPhase(pos);
}

function deserializeVecImportDeclarationSpecifier(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeImportDeclarationSpecifier(pos));
    pos += 16;
  }
  return arr;
}

function deserializeOptionVecImportDeclarationSpecifier(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeVecImportDeclarationSpecifier(pos);
}

function deserializeBoxWithClause(pos) {
  return deserializeWithClause(int32[pos >> 2]);
}

function deserializeOptionBoxWithClause(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxWithClause(pos);
}

function deserializeBoxImportSpecifier(pos) {
  return deserializeImportSpecifier(int32[pos >> 2]);
}

function deserializeBoxImportDefaultSpecifier(pos) {
  return deserializeImportDefaultSpecifier(int32[pos >> 2]);
}

function deserializeBoxImportNamespaceSpecifier(pos) {
  return deserializeImportNamespaceSpecifier(int32[pos >> 2]);
}

function deserializeVecImportAttribute(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 120;
  for (; pos !== endPos; ) {
    arr.push(deserializeImportAttribute(pos));
    pos += 120;
  }
  return arr;
}

function deserializeOptionDeclaration(pos) {
  return uint8[pos] === 31 ? null : deserializeDeclaration(pos);
}

function deserializeVecExportSpecifier(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 7);
  for (; pos !== endPos; ) {
    arr.push(deserializeExportSpecifier(pos));
    pos += 128;
  }
  return arr;
}

function deserializeOptionStringLiteral(pos) {
  return uint8[pos + 12] === 2 ? null : deserializeStringLiteral(pos);
}

function deserializeOptionModuleExportName(pos) {
  return uint8[pos] === 3 ? null : deserializeModuleExportName(pos);
}

function deserializeF64(pos) {
  return float64[pos >> 3];
}

function deserializeU8(pos) {
  return uint8[pos];
}

function deserializeBoxJSXOpeningElement(pos) {
  return deserializeJSXOpeningElement(int32[pos >> 2]);
}

function deserializeVecJSXChild(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeJSXChild(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxJSXClosingElement(pos) {
  return deserializeJSXClosingElement(int32[pos >> 2]);
}

function deserializeOptionBoxJSXClosingElement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxJSXClosingElement(pos);
}

function deserializeVecJSXAttributeItem(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeJSXAttributeItem(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxJSXIdentifier(pos) {
  return deserializeJSXIdentifier(int32[pos >> 2]);
}

function deserializeBoxJSXNamespacedName(pos) {
  return deserializeJSXNamespacedName(int32[pos >> 2]);
}

function deserializeBoxJSXMemberExpression(pos) {
  return deserializeJSXMemberExpression(int32[pos >> 2]);
}

function deserializeBoxJSXEmptyExpression(pos) {
  return deserializeJSXEmptyExpression(int32[pos >> 2]);
}

function deserializeBoxJSXAttribute(pos) {
  return deserializeJSXAttribute(int32[pos >> 2]);
}

function deserializeBoxJSXSpreadAttribute(pos) {
  return deserializeJSXSpreadAttribute(int32[pos >> 2]);
}

function deserializeOptionJSXAttributeValue(pos) {
  return uint8[pos] === 4 ? null : deserializeJSXAttributeValue(pos);
}

function deserializeBoxJSXExpressionContainer(pos) {
  return deserializeJSXExpressionContainer(int32[pos >> 2]);
}

function deserializeBoxJSXText(pos) {
  return deserializeJSXText(int32[pos >> 2]);
}

function deserializeBoxJSXSpreadChild(pos) {
  return deserializeJSXSpreadChild(int32[pos >> 2]);
}

function deserializeVecTSEnumMember(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 48;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSEnumMember(pos));
    pos += 48;
  }
  return arr;
}

function deserializeBoxTSAnyKeyword(pos) {
  return deserializeTSAnyKeyword(int32[pos >> 2]);
}

function deserializeBoxTSBigIntKeyword(pos) {
  return deserializeTSBigIntKeyword(int32[pos >> 2]);
}

function deserializeBoxTSBooleanKeyword(pos) {
  return deserializeTSBooleanKeyword(int32[pos >> 2]);
}

function deserializeBoxTSIntrinsicKeyword(pos) {
  return deserializeTSIntrinsicKeyword(int32[pos >> 2]);
}

function deserializeBoxTSNeverKeyword(pos) {
  return deserializeTSNeverKeyword(int32[pos >> 2]);
}

function deserializeBoxTSNullKeyword(pos) {
  return deserializeTSNullKeyword(int32[pos >> 2]);
}

function deserializeBoxTSNumberKeyword(pos) {
  return deserializeTSNumberKeyword(int32[pos >> 2]);
}

function deserializeBoxTSObjectKeyword(pos) {
  return deserializeTSObjectKeyword(int32[pos >> 2]);
}

function deserializeBoxTSStringKeyword(pos) {
  return deserializeTSStringKeyword(int32[pos >> 2]);
}

function deserializeBoxTSSymbolKeyword(pos) {
  return deserializeTSSymbolKeyword(int32[pos >> 2]);
}

function deserializeBoxTSUndefinedKeyword(pos) {
  return deserializeTSUndefinedKeyword(int32[pos >> 2]);
}

function deserializeBoxTSUnknownKeyword(pos) {
  return deserializeTSUnknownKeyword(int32[pos >> 2]);
}

function deserializeBoxTSVoidKeyword(pos) {
  return deserializeTSVoidKeyword(int32[pos >> 2]);
}

function deserializeBoxTSArrayType(pos) {
  return deserializeTSArrayType(int32[pos >> 2]);
}

function deserializeBoxTSConditionalType(pos) {
  return deserializeTSConditionalType(int32[pos >> 2]);
}

function deserializeBoxTSConstructorType(pos) {
  return deserializeTSConstructorType(int32[pos >> 2]);
}

function deserializeBoxTSFunctionType(pos) {
  return deserializeTSFunctionType(int32[pos >> 2]);
}

function deserializeBoxTSImportType(pos) {
  return deserializeTSImportType(int32[pos >> 2]);
}

function deserializeBoxTSIndexedAccessType(pos) {
  return deserializeTSIndexedAccessType(int32[pos >> 2]);
}

function deserializeBoxTSInferType(pos) {
  return deserializeTSInferType(int32[pos >> 2]);
}

function deserializeBoxTSIntersectionType(pos) {
  return deserializeTSIntersectionType(int32[pos >> 2]);
}

function deserializeBoxTSLiteralType(pos) {
  return deserializeTSLiteralType(int32[pos >> 2]);
}

function deserializeBoxTSMappedType(pos) {
  return deserializeTSMappedType(int32[pos >> 2]);
}

function deserializeBoxTSNamedTupleMember(pos) {
  return deserializeTSNamedTupleMember(int32[pos >> 2]);
}

function deserializeBoxTSTemplateLiteralType(pos) {
  return deserializeTSTemplateLiteralType(int32[pos >> 2]);
}

function deserializeBoxTSThisType(pos) {
  return deserializeTSThisType(int32[pos >> 2]);
}

function deserializeBoxTSTupleType(pos) {
  return deserializeTSTupleType(int32[pos >> 2]);
}

function deserializeBoxTSTypeLiteral(pos) {
  return deserializeTSTypeLiteral(int32[pos >> 2]);
}

function deserializeBoxTSTypeOperator(pos) {
  return deserializeTSTypeOperator(int32[pos >> 2]);
}

function deserializeBoxTSTypePredicate(pos) {
  return deserializeTSTypePredicate(int32[pos >> 2]);
}

function deserializeBoxTSTypeQuery(pos) {
  return deserializeTSTypeQuery(int32[pos >> 2]);
}

function deserializeBoxTSTypeReference(pos) {
  return deserializeTSTypeReference(int32[pos >> 2]);
}

function deserializeBoxTSUnionType(pos) {
  return deserializeTSUnionType(int32[pos >> 2]);
}

function deserializeBoxTSParenthesizedType(pos) {
  return deserializeTSParenthesizedType(int32[pos >> 2]);
}

function deserializeBoxJSDocNullableType(pos) {
  return deserializeJSDocNullableType(int32[pos >> 2]);
}

function deserializeBoxJSDocNonNullableType(pos) {
  return deserializeJSDocNonNullableType(int32[pos >> 2]);
}

function deserializeBoxJSDocUnknownType(pos) {
  return deserializeJSDocUnknownType(int32[pos >> 2]);
}

function deserializeVecTSType(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeTSType(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecTSTupleElement(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeTSTupleElement(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSOptionalType(pos) {
  return deserializeTSOptionalType(int32[pos >> 2]);
}

function deserializeBoxTSRestType(pos) {
  return deserializeTSRestType(int32[pos >> 2]);
}

function deserializeBoxTSQualifiedName(pos) {
  return deserializeTSQualifiedName(int32[pos >> 2]);
}

function deserializeOptionTSType(pos) {
  return uint8[pos] === 38 ? null : deserializeTSType(pos);
}

function deserializeVecTSTypeParameter(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 80;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSTypeParameter(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecTSInterfaceHeritage(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 40;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSInterfaceHeritage(pos));
    pos += 40;
  }
  return arr;
}

function deserializeBoxTSInterfaceBody(pos) {
  return deserializeTSInterfaceBody(int32[pos >> 2]);
}

function deserializeVecTSSignature(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeTSSignature(pos));
    pos += 16;
  }
  return arr;
}

function deserializeBoxTSPropertySignature(pos) {
  return deserializeTSPropertySignature(int32[pos >> 2]);
}

function deserializeBoxTSCallSignatureDeclaration(pos) {
  return deserializeTSCallSignatureDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSConstructSignatureDeclaration(pos) {
  return deserializeTSConstructSignatureDeclaration(int32[pos >> 2]);
}

function deserializeBoxTSMethodSignature(pos) {
  return deserializeTSMethodSignature(int32[pos >> 2]);
}

function deserializeVecTSIndexSignatureName(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 40;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSIndexSignatureName(pos));
    pos += 40;
  }
  return arr;
}

function deserializeOptionTSModuleDeclarationBody(pos) {
  return uint8[pos] === 2 ? null : deserializeTSModuleDeclarationBody(pos);
}

function deserializeBoxTSModuleBlock(pos) {
  return deserializeTSModuleBlock(int32[pos >> 2]);
}

function deserializeBoxTSTypeParameter(pos) {
  return deserializeTSTypeParameter(int32[pos >> 2]);
}

function deserializeOptionBoxObjectExpression(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBoxObjectExpression(pos);
}

function deserializeOptionTSImportTypeQualifier(pos) {
  return uint8[pos] === 2 ? null : deserializeTSImportTypeQualifier(pos);
}

function deserializeBoxTSImportTypeQualifiedName(pos) {
  return deserializeTSImportTypeQualifiedName(int32[pos >> 2]);
}

function deserializeOptionTSMappedTypeModifierOperator(pos) {
  return uint8[pos] === 3 ? null : deserializeTSMappedTypeModifierOperator(pos);
}

function deserializeBoxTSExternalModuleReference(pos) {
  return deserializeTSExternalModuleReference(int32[pos >> 2]);
}

function deserializeI32(pos) {
  return int32[pos >> 2];
}

function deserializeOptionNameSpan(pos) {
  return int32[(pos >> 2) + 2] === 0 && int32[(pos >> 2) + 3] === 0
    ? null
    : deserializeNameSpan(pos);
}

function deserializeVecError(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 80;
  for (; pos !== endPos; ) {
    arr.push(deserializeError(pos));
    pos += 80;
  }
  return arr;
}

function deserializeVecErrorLabel(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 24;
  for (; pos !== endPos; ) {
    arr.push(deserializeErrorLabel(pos));
    pos += 24;
  }
  return arr;
}

function deserializeVecStaticImport(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 56;
  for (; pos !== endPos; ) {
    arr.push(deserializeStaticImport(pos));
    pos += 56;
  }
  return arr;
}

function deserializeVecStaticExport(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 5);
  for (; pos !== endPos; ) {
    arr.push(deserializeStaticExport(pos));
    pos += 32;
  }
  return arr;
}

function deserializeVecDynamicImport(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 4);
  for (; pos !== endPos; ) {
    arr.push(deserializeDynamicImport(pos));
    pos += 16;
  }
  return arr;
}

function deserializeVecSpan(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + (int32[pos32 + 2] << 3);
  for (; pos !== endPos; ) {
    arr.push(deserializeSpan(pos));
    pos += 8;
  }
  return arr;
}

function deserializeVecImportEntry(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 96;
  for (; pos !== endPos; ) {
    arr.push(deserializeImportEntry(pos));
    pos += 96;
  }
  return arr;
}

function deserializeVecExportEntry(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = int32[pos32];
  let endPos = pos + int32[pos32 + 2] * 144;
  for (; pos !== endPos; ) {
    arr.push(deserializeExportEntry(pos));
    pos += 144;
  }
  return arr;
}
