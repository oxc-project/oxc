// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8,
  uint32,
  float64,
  sourceText,
  sourceIsAscii,
  sourceByteLen,
  parent = null;

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
  let end = deserializeU32(pos + 4),
    program = (parent = {
      type: "Program",
      body: null,
      sourceType: deserializeModuleKind(pos + 125),
      hashbang: null,
      start: 0,
      end,
      range: [0, end],
      parent: null,
    });
  program.hashbang = deserializeOptionHashbang(pos + 48);
  let body = (program.body = deserializeVecDirective(pos + 72));
  body.push(...deserializeVecStatement(pos + 96));
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: null,
      name: deserializeStr(pos + 8),
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeIdentifierReference(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: null,
      name: deserializeStr(pos + 8),
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeBindingIdentifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: null,
      name: deserializeStr(pos + 8),
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeLabelIdentifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: null,
      name: deserializeStr(pos + 8),
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeThisExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "ThisExpression",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeArrayExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ArrayExpression",
      elements: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.elements = deserializeVecArrayExpressionElement(pos + 8);
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
      return deserializeElision(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ArrayExpressionElement`);
  }
}

function deserializeElision(pos) {
  return null;
}

function deserializeObjectExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ObjectExpression",
      properties: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.properties = deserializeVecObjectPropertyKind(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: deserializePropertyKind(pos + 40),
      key: null,
      value: null,
      method: deserializeBool(pos + 41),
      shorthand: deserializeBool(pos + 42),
      computed: deserializeBool(pos + 43),
      optional: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.key = deserializePropertyKey(pos + 8);
  node.value = deserializeExpression(pos + 24);
  node.optional = false;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TemplateLiteral",
      quasis: null,
      expressions: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.quasis = deserializeVecTemplateElement(pos + 8);
  node.expressions = deserializeVecExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTaggedTemplateExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TaggedTemplateExpression",
      tag: null,
      typeArguments: null,
      quasi: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.tag = deserializeExpression(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  node.quasi = deserializeTemplateLiteral(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTemplateElement(pos) {
  let tail = deserializeBool(pos + 40),
    start = deserializeU32(pos) - 1,
    end = deserializeU32(pos + 4) + 2 - tail,
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 40),
      computed: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 8);
  node.property = deserializeExpression(pos + 24);
  node.computed = true;
  parent = previousParent;
  return node;
}

function deserializeStaticMemberExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 48),
      computed: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 8);
  node.property = deserializeIdentifierName(pos + 24);
  node.computed = false;
  parent = previousParent;
  return node;
}

function deserializePrivateFieldExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: deserializeBool(pos + 48),
      computed: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 8);
  node.property = deserializePrivateIdentifier(pos + 24);
  node.computed = false;
  parent = previousParent;
  return node;
}

function deserializeCallExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "CallExpression",
      callee: null,
      typeArguments: null,
      arguments: null,
      optional: deserializeBool(pos + 56),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.callee = deserializeExpression(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  node.arguments = deserializeVecArgument(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeNewExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "NewExpression",
      callee: null,
      typeArguments: null,
      arguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.callee = deserializeExpression(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  node.arguments = deserializeVecArgument(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeMetaProperty(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "MetaProperty",
      meta: null,
      property: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.meta = deserializeIdentifierName(pos + 8);
  node.property = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSpreadElement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "SpreadElement",
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "UpdateExpression",
      operator: deserializeUpdateOperator(pos + 24),
      prefix: deserializeBool(pos + 25),
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeSimpleAssignmentTarget(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeUnaryExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "UnaryExpression",
      operator: deserializeUnaryOperator(pos + 24),
      argument: null,
      prefix: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 8);
  node.prefix = true;
  parent = previousParent;
  return node;
}

function deserializeBinaryExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "BinaryExpression",
      left: null,
      operator: deserializeBinaryOperator(pos + 40),
      right: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeExpression(pos + 8);
  node.right = deserializeExpression(pos + 24);
  parent = previousParent;
  return node;
}

function deserializePrivateInExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "BinaryExpression",
      left: null,
      operator: null,
      right: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializePrivateIdentifier(pos + 8);
  node.operator = "in";
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeLogicalExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "LogicalExpression",
      left: null,
      operator: deserializeLogicalOperator(pos + 40),
      right: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeExpression(pos + 8);
  node.right = deserializeExpression(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeConditionalExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ConditionalExpression",
      test: null,
      consequent: null,
      alternate: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.test = deserializeExpression(pos + 8);
  node.consequent = deserializeExpression(pos + 24);
  node.alternate = deserializeExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeAssignmentExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "AssignmentExpression",
      operator: deserializeAssignmentOperator(pos + 40),
      left: null,
      right: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeAssignmentTarget(pos + 8);
  node.right = deserializeExpression(pos + 24);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ArrayPattern",
      decorators: null,
      elements: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 8),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 32);
  rest !== null && elements.push(rest);
  node.decorators = [];
  node.elements = elements;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeObjectAssignmentTarget(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ObjectPattern",
      decorators: null,
      properties: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    properties = deserializeVecAssignmentTargetProperty(pos + 8),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 32);
  rest !== null && properties.push(rest);
  node.decorators = [];
  node.properties = properties;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetRest(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "RestElement",
      decorators: null,
      argument: null,
      optional: null,
      typeAnnotation: null,
      value: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.argument = deserializeAssignmentTarget(pos + 8);
  node.optional = false;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "AssignmentPattern",
      decorators: null,
      left: null,
      right: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.left = deserializeAssignmentTarget(pos + 8);
  node.right = deserializeExpression(pos + 24);
  node.optional = false;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: null,
      key: null,
      value: null,
      method: null,
      shorthand: null,
      computed: null,
      optional: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    key = deserializeIdentifierReference(pos + 8),
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
    init = deserializeOptionExpression(pos + 40);
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
  node.kind = "init";
  node.key = key;
  node.value = value;
  node.method = false;
  node.shorthand = true;
  node.computed = false;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: null,
      key: null,
      value: null,
      method: null,
      shorthand: null,
      computed: deserializeBool(pos + 40),
      optional: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.kind = "init";
  node.key = deserializePropertyKey(pos + 8);
  node.value = deserializeAssignmentTargetMaybeDefault(pos + 24);
  node.method = false;
  node.shorthand = false;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeSequenceExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "SequenceExpression",
      expressions: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expressions = deserializeVecExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeSuper(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "Super",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeAwaitExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "AwaitExpression",
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeChainExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ChainExpression",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeChainElement(pos + 8);
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
      start: (start = deserializeU32(pos)),
      end: (end = deserializeU32(pos + 4)),
      range: [start, end],
      parent,
    };
    node.expression = deserializeExpression(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ExpressionStatement",
      expression: null,
      directive: deserializeStr(pos + 56),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeStringLiteral(pos + 8);
  parent = previousParent;
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
    range: [start, end],
    parent,
  };
}

function deserializeBlockStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "BlockStatement",
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.body = deserializeVecStatement(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "VariableDeclaration",
      kind: deserializeVariableDeclarationKind(pos + 32),
      declarations: null,
      declare: deserializeBool(pos + 33),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.declarations = deserializeVecVariableDeclarator(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "VariableDeclarator",
      id: null,
      init: null,
      definite: deserializeBool(pos + 57),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingPattern(pos + 8);
  node.init = deserializeOptionExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeEmptyStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "EmptyStatement",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeExpressionStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ExpressionStatement",
      expression: null,
      directive: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeIfStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "IfStatement",
      test: null,
      consequent: null,
      alternate: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.test = deserializeExpression(pos + 8);
  node.consequent = deserializeStatement(pos + 24);
  node.alternate = deserializeOptionStatement(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeDoWhileStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "DoWhileStatement",
      body: null,
      test: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.body = deserializeStatement(pos + 8);
  node.test = deserializeExpression(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeWhileStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "WhileStatement",
      test: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.test = deserializeExpression(pos + 8);
  node.body = deserializeStatement(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeForStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ForStatement",
      init: null,
      test: null,
      update: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.init = deserializeOptionForStatementInit(pos + 8);
  node.test = deserializeOptionExpression(pos + 24);
  node.update = deserializeOptionExpression(pos + 40);
  node.body = deserializeStatement(pos + 56);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ForInStatement",
      left: null,
      right: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeForStatementLeft(pos + 8);
  node.right = deserializeExpression(pos + 24);
  node.body = deserializeStatement(pos + 40);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ForOfStatement",
      await: deserializeBool(pos + 60),
      left: null,
      right: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeForStatementLeft(pos + 8);
  node.right = deserializeExpression(pos + 24);
  node.body = deserializeStatement(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeContinueStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ContinueStatement",
      label: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.label = deserializeOptionLabelIdentifier(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeBreakStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "BreakStatement",
      label: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.label = deserializeOptionLabelIdentifier(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeReturnStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ReturnStatement",
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeOptionExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeWithStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "WithStatement",
      object: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.object = deserializeExpression(pos + 8);
  node.body = deserializeStatement(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeSwitchStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "SwitchStatement",
      discriminant: null,
      cases: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.discriminant = deserializeExpression(pos + 8);
  node.cases = deserializeVecSwitchCase(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeSwitchCase(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "SwitchCase",
      test: null,
      consequent: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.test = deserializeOptionExpression(pos + 8);
  node.consequent = deserializeVecStatement(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeLabeledStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "LabeledStatement",
      label: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.label = deserializeLabelIdentifier(pos + 8);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeThrowStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ThrowStatement",
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTryStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TryStatement",
      block: null,
      handler: null,
      finalizer: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.block = deserializeBoxBlockStatement(pos + 8);
  node.handler = deserializeOptionBoxCatchClause(pos + 16);
  node.finalizer = deserializeOptionBoxBlockStatement(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeCatchClause(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "CatchClause",
      param: null,
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.param = deserializeOptionCatchParameter(pos + 8);
  node.body = deserializeBoxBlockStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeCatchParameter(pos) {
  return deserializeBindingPattern(pos + 8);
}

function deserializeDebuggerStatement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "DebuggerStatement",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeBindingPattern(pos) {
  let pattern = deserializeBindingPatternKind(pos);
  {
    let previousParent = parent;
    parent = pattern;
    pattern.optional = deserializeBool(pos + 24);
    pattern.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 16);
    parent = previousParent;
  }
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "AssignmentPattern",
      decorators: null,
      left: null,
      right: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.left = deserializeBindingPattern(pos + 8);
  node.right = deserializeExpression(pos + 40);
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeObjectPattern(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ObjectPattern",
      decorators: null,
      properties: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    properties = deserializeVecBindingProperty(pos + 8),
    rest = deserializeOptionBoxBindingRestElement(pos + 32);
  rest !== null && properties.push(rest);
  node.decorators = [];
  node.properties = properties;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeBindingProperty(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: null,
      key: null,
      value: null,
      method: null,
      shorthand: deserializeBool(pos + 56),
      computed: deserializeBool(pos + 57),
      optional: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.kind = "init";
  node.key = deserializePropertyKey(pos + 8);
  node.value = deserializeBindingPattern(pos + 24);
  node.method = false;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeArrayPattern(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ArrayPattern",
      decorators: null,
      elements: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    elements = deserializeVecOptionBindingPattern(pos + 8),
    rest = deserializeOptionBoxBindingRestElement(pos + 32);
  rest !== null && elements.push(rest);
  node.decorators = [];
  node.elements = elements;
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeBindingRestElement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "RestElement",
      decorators: null,
      argument: null,
      optional: null,
      typeAnnotation: null,
      value: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.argument = deserializeBindingPattern(pos + 8);
  node.optional = false;
  parent = previousParent;
  return node;
}

function deserializeFunction(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: deserializeFunctionType(pos + 84),
      id: null,
      generator: deserializeBool(pos + 85),
      async: deserializeBool(pos + 86),
      declare: deserializeBool(pos + 87),
      typeParameters: null,
      params: null,
      returnType: null,
      body: null,
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 56);
  {
    let thisParam = deserializeOptionBoxTSThisParameter(pos + 48);
    thisParam !== null && params.unshift(thisParam);
  }
  node.id = deserializeOptionBindingIdentifier(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 40);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 64);
  node.body = deserializeOptionBoxFunctionBody(pos + 72);
  node.expression = false;
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
  let params = deserializeVecFormalParameter(pos + 8);
  if (uint32[(pos + 32) >> 2] !== 0 && uint32[(pos + 36) >> 2] !== 0) {
    pos = uint32[(pos + 32) >> 2];
    let start,
      end,
      previousParent = parent,
      rest = (parent = {
        type: "RestElement",
        decorators: [],
        argument: null,
        optional: deserializeBool(pos + 32),
        typeAnnotation: null,
        value: null,
        start: (start = deserializeU32(pos)),
        end: (end = deserializeU32(pos + 4)),
        range: [start, end],
        parent,
      });
    rest.argument = deserializeBindingPatternKind(pos + 8);
    rest.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
    params.push(rest);
    parent = previousParent;
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param;
  {
    let accessibility = deserializeOptionTSAccessibility(pos + 64),
      readonly = deserializeBool(pos + 65),
      override = deserializeBool(pos + 66),
      previousParent = parent;
    if (accessibility === null && !readonly && !override) {
      param = parent = deserializeBindingPatternKind(pos + 32);
      param.decorators = deserializeVecDecorator(pos + 8);
      param.optional = deserializeBool(pos + 56);
      param.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 48);
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
        start: (start = deserializeU32(pos)),
        end: (end = deserializeU32(pos + 4)),
        range: [start, end],
        parent,
      };
      param.decorators = deserializeVecDecorator(pos + 8);
      param.parameter = deserializeBindingPattern(pos + 32);
    }
    parent = previousParent;
  }
  return param;
}

function deserializeFunctionBody(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "BlockStatement",
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 32));
  node.body = body;
  parent = previousParent;
  return node;
}

function deserializeArrowFunctionExpression(pos) {
  let expression = deserializeBool(pos + 44),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ArrowFunctionExpression",
      expression,
      async: deserializeBool(pos + 45),
      typeParameters: null,
      params: null,
      returnType: null,
      body: null,
      id: null,
      generator: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    body = deserializeBoxFunctionBody(pos + 32);
  if (expression === true) {
    body = body.body[0].expression;
    body.parent = parent;
  }
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = deserializeBoxFormalParameters(pos + 16);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  node.body = body;
  node.generator = false;
  parent = previousParent;
  return node;
}

function deserializeYieldExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "YieldExpression",
      delegate: deserializeBool(pos + 24),
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeOptionExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeClass(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: deserializeClassType(pos + 132),
      decorators: null,
      id: null,
      typeParameters: null,
      superClass: null,
      superTypeArguments: null,
      implements: null,
      body: null,
      abstract: deserializeBool(pos + 133),
      declare: deserializeBool(pos + 134),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 8);
  node.id = deserializeOptionBindingIdentifier(pos + 32);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 64);
  node.superClass = deserializeOptionExpression(pos + 72);
  node.superTypeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 88);
  node.implements = deserializeVecTSClassImplements(pos + 96);
  node.body = deserializeBoxClassBody(pos + 120);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ClassBody",
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.body = deserializeVecClassElement(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: deserializeMethodDefinitionType(pos + 56),
      decorators: null,
      key: null,
      value: null,
      kind: deserializeMethodDefinitionKind(pos + 57),
      computed: deserializeBool(pos + 58),
      static: deserializeBool(pos + 59),
      override: deserializeBool(pos + 60),
      optional: deserializeBool(pos + 61),
      accessibility: deserializeOptionTSAccessibility(pos + 62),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 8);
  node.key = deserializePropertyKey(pos + 32);
  node.value = deserializeBoxFunction(pos + 48);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: deserializePropertyDefinitionType(pos + 72),
      decorators: null,
      key: null,
      typeAnnotation: null,
      value: null,
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
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 8);
  node.key = deserializePropertyKey(pos + 32);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 48);
  node.value = deserializeOptionExpression(pos + 56);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "PrivateIdentifier",
    name: deserializeStr(pos + 8),
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeStaticBlock(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "StaticBlock",
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.body = deserializeVecStatement(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: deserializeAccessorPropertyType(pos + 72),
      decorators: null,
      key: null,
      typeAnnotation: null,
      value: null,
      computed: deserializeBool(pos + 73),
      static: deserializeBool(pos + 74),
      override: deserializeBool(pos + 75),
      definite: deserializeBool(pos + 76),
      accessibility: deserializeOptionTSAccessibility(pos + 77),
      declare: null,
      optional: null,
      readonly: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 8);
  node.key = deserializePropertyKey(pos + 32);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 48);
  node.value = deserializeOptionExpression(pos + 56);
  node.declare = false;
  node.optional = false;
  node.readonly = false;
  parent = previousParent;
  return node;
}

function deserializeImportExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ImportExpression",
      source: null,
      options: null,
      phase: deserializeOptionImportPhase(pos + 40),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.source = deserializeExpression(pos + 8);
  node.options = deserializeOptionExpression(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeImportDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ImportDeclaration",
      specifiers: null,
      source: null,
      phase: deserializeOptionImportPhase(pos + 88),
      attributes: null,
      importKind: deserializeImportOrExportKind(pos + 89),
      start,
      end,
      range: [start, end],
      parent,
    }),
    specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 8);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 80);
  node.specifiers = specifiers;
  node.source = deserializeStringLiteral(pos + 32);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ImportSpecifier",
      imported: null,
      local: null,
      importKind: deserializeImportOrExportKind(pos + 96),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.imported = deserializeModuleExportName(pos + 8);
  node.local = deserializeBindingIdentifier(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeImportDefaultSpecifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ImportDefaultSpecifier",
      local: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.local = deserializeBindingIdentifier(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeImportNamespaceSpecifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ImportNamespaceSpecifier",
      local: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.local = deserializeBindingIdentifier(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 8) };
}

function deserializeImportAttribute(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ImportAttribute",
      key: null,
      value: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.key = deserializeImportAttributeKey(pos + 8);
  node.value = deserializeStringLiteral(pos + 64);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      exportKind: deserializeImportOrExportKind(pos + 104),
      attributes: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    withClause = deserializeOptionBoxWithClause(pos + 96);
  node.declaration = deserializeOptionDeclaration(pos + 8);
  node.specifiers = deserializeVecExportSpecifier(pos + 24);
  node.source = deserializeOptionStringLiteral(pos + 48);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportDefaultDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ExportDefaultDeclaration",
      declaration: null,
      exportKind: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.declaration = deserializeExportDefaultDeclarationKind(pos + 8);
  node.exportKind = "value";
  parent = previousParent;
  return node;
}

function deserializeExportAllDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ExportAllDeclaration",
      exported: null,
      source: null,
      attributes: null,
      exportKind: deserializeImportOrExportKind(pos + 120),
      start,
      end,
      range: [start, end],
      parent,
    }),
    withClause = deserializeOptionBoxWithClause(pos + 112);
  node.exported = deserializeOptionModuleExportName(pos + 8);
  node.source = deserializeStringLiteral(pos + 64);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportSpecifier(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "ExportSpecifier",
      local: null,
      exported: null,
      exportKind: deserializeImportOrExportKind(pos + 120),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.local = deserializeModuleExportName(pos + 8);
  node.exported = deserializeModuleExportName(pos + 64);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "V8IntrinsicExpression",
      name: null,
      arguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.name = deserializeIdentifierName(pos + 8);
  node.arguments = deserializeVecArgument(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeBooleanLiteral(pos) {
  let value = deserializeBool(pos + 8),
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.raw = start === 0 && end === 0 ? null : "null";
  parent = previousParent;
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
    range: [start, end],
    parent,
  };
}

function deserializeStringLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw: deserializeOptionStr(pos + 24),
      start,
      end,
      range: [start, end],
      parent,
    }),
    value = deserializeStr(pos + 8);
  deserializeBool(pos + 40) &&
    (value = value.replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16))));
  node.value = value;
  parent = previousParent;
  return node;
}

function deserializeBigIntLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw: deserializeOptionStr(pos + 24),
      bigint: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    bigint = deserializeStr(pos + 8);
  node.value = BigInt(bigint);
  node.bigint = bigint;
  parent = previousParent;
  return node;
}

function deserializeRegExpLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw: deserializeOptionStr(pos + 40),
      regex: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    regex = deserializeRegExp(pos + 8),
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXElement",
      openingElement: null,
      children: null,
      closingElement: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    closingElement = deserializeOptionBoxJSXClosingElement(pos + 40),
    openingElement = deserializeBoxJSXOpeningElement(pos + 8);
  closingElement === null && (openingElement.selfClosing = true);
  node.openingElement = openingElement;
  node.children = deserializeVecJSXChild(pos + 16);
  node.closingElement = closingElement;
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningElement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXOpeningElement",
      name: null,
      typeArguments: null,
      attributes: null,
      selfClosing: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.name = deserializeJSXElementName(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  node.attributes = deserializeVecJSXAttributeItem(pos + 32);
  node.selfClosing = false;
  parent = previousParent;
  return node;
}

function deserializeJSXClosingElement(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXClosingElement",
      name: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.name = deserializeJSXElementName(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeJSXFragment(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXFragment",
      openingFragment: null,
      children: null,
      closingFragment: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.openingFragment = deserializeJSXOpeningFragment(pos + 8);
  node.children = deserializeVecJSXChild(pos + 16);
  node.closingFragment = deserializeJSXClosingFragment(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningFragment(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXOpeningFragment",
      start,
      end,
      range: [start, end],
      parent,
    });
  parent = previousParent;
  return node;
}

function deserializeJSXClosingFragment(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "JSXClosingFragment",
    start,
    end,
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXNamespacedName",
      namespace: null,
      name: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.namespace = deserializeJSXIdentifier(pos + 8);
  node.name = deserializeJSXIdentifier(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeJSXMemberExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXMemberExpression",
      object: null,
      property: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.object = deserializeJSXMemberExpressionObject(pos + 8);
  node.property = deserializeJSXIdentifier(pos + 24);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXExpressionContainer",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeJSXExpression(pos + 8);
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
      return deserializeJSXEmptyExpression(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXExpression`);
  }
}

function deserializeJSXEmptyExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "JSXEmptyExpression",
    start,
    end,
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXAttribute",
      name: null,
      value: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.name = deserializeJSXAttributeName(pos + 8);
  node.value = deserializeOptionJSXAttributeValue(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeJSXSpreadAttribute(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXSpreadAttribute",
      argument: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.argument = deserializeExpression(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "JSXIdentifier",
    name: deserializeStr(pos + 8),
    start,
    end,
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "JSXSpreadChild",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  parent = previousParent;
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
    range: [start, end],
    parent,
  };
}

function deserializeTSThisParameter(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: null,
      name: null,
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.name = "this";
  node.optional = false;
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSEnumDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSEnumDeclaration",
      id: null,
      body: null,
      const: deserializeBool(pos + 76),
      declare: deserializeBool(pos + 77),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 8);
  node.body = deserializeTSEnumBody(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTSEnumBody(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSEnumBody",
      members: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.members = deserializeVecTSEnumMember(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSEnumMember(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSEnumMember",
      id: null,
      initializer: null,
      computed: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeTSEnumMemberName(pos + 8);
  node.initializer = deserializeOptionExpression(pos + 24);
  node.computed = deserializeU8(pos + 8) > 1;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeAnnotation",
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSLiteralType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSLiteralType",
      literal: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.literal = deserializeTSLiteral(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSConditionalType",
      checkType: null,
      extendsType: null,
      trueType: null,
      falseType: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.checkType = deserializeTSType(pos + 8);
  node.extendsType = deserializeTSType(pos + 24);
  node.trueType = deserializeTSType(pos + 40);
  node.falseType = deserializeTSType(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeTSUnionType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSUnionType",
      types: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.types = deserializeVecTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSIntersectionType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSIntersectionType",
      types: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.types = deserializeVecTSType(pos + 8);
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
      start: (start = deserializeU32(pos)),
      end: (end = deserializeU32(pos + 4)),
      range: [start, end],
      parent,
    };
    node.typeAnnotation = deserializeTSType(pos + 8);
    parent = previousParent;
  }
  return node;
}

function deserializeTSTypeOperator(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeOperator",
      operator: deserializeTSTypeOperatorOperator(pos + 24),
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSArrayType",
      elementType: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.elementType = deserializeTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSIndexedAccessType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSIndexedAccessType",
      objectType: null,
      indexType: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.objectType = deserializeTSType(pos + 8);
  node.indexType = deserializeTSType(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSTupleType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTupleType",
      elementTypes: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.elementTypes = deserializeVecTSTupleElement(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSNamedTupleMember(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSNamedTupleMember",
      label: null,
      elementType: null,
      optional: deserializeBool(pos + 48),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.label = deserializeIdentifierName(pos + 8);
  node.elementType = deserializeTSTupleElement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSOptionalType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSOptionalType",
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSRestType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSRestType",
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSAnyKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSStringKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSStringKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSBooleanKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSBooleanKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSNumberKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSNumberKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSNeverKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSNeverKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSIntrinsicKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSUnknownKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSUnknownKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSNullKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSNullKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSUndefinedKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSUndefinedKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSVoidKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSVoidKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSSymbolKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSSymbolKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSThisType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSThisType",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSObjectKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSObjectKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSBigIntKeyword(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSBigIntKeyword",
    start,
    end,
    range: [start, end],
    parent,
  };
}

function deserializeTSTypeReference(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeReference",
      typeName: null,
      typeArguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeName = deserializeTSTypeName(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeTSTypeName(pos + 8);
  node.right = deserializeIdentifierName(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterInstantiation(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeParameterInstantiation",
      params: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.params = deserializeVecTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameter(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeParameter",
      name: null,
      constraint: null,
      default: null,
      in: deserializeBool(pos + 72),
      out: deserializeBool(pos + 73),
      const: deserializeBool(pos + 74),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.name = deserializeBindingIdentifier(pos + 8);
  node.constraint = deserializeOptionTSType(pos + 40);
  node.default = deserializeOptionTSType(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeParameterDeclaration",
      params: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.params = deserializeVecTSTypeParameter(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAliasDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeAliasDeclaration",
      id: null,
      typeParameters: null,
      typeAnnotation: null,
      declare: deserializeBool(pos + 68),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 40);
  node.typeAnnotation = deserializeTSType(pos + 48);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSClassImplements",
      expression: null,
      typeArguments: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    expression = deserializeTSTypeName(pos + 8);
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
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSInterfaceDeclaration",
      id: null,
      typeParameters: null,
      extends: null,
      body: null,
      declare: deserializeBool(pos + 84),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 40);
  node.extends = deserializeVecTSInterfaceHeritage(pos + 48);
  node.body = deserializeBoxTSInterfaceBody(pos + 72);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceBody(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSInterfaceBody",
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.body = deserializeVecTSSignature(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSPropertySignature(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
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
      range: [start, end],
      parent,
    });
  node.key = deserializePropertyKey(pos + 8);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  node.static = false;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSIndexSignature",
      parameters: null,
      typeAnnotation: null,
      readonly: deserializeBool(pos + 40),
      static: deserializeBool(pos + 41),
      accessibility: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.parameters = deserializeVecTSIndexSignatureName(pos + 8);
  node.typeAnnotation = deserializeBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSCallSignatureDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSCallSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 24),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 16);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
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
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 40),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 32);
  thisParam !== null && params.unshift(thisParam);
  node.key = deserializePropertyKey(pos + 8);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 24);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 48);
  node.readonly = false;
  node.static = false;
  parent = previousParent;
  return node;
}

function deserializeTSConstructSignatureDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSConstructSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = deserializeBoxFormalParameters(pos + 16);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSIndexSignatureName(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: null,
      name: deserializeStr(pos + 8),
      optional: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.decorators = [];
  node.optional = false;
  node.typeAnnotation = deserializeBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceHeritage(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSInterfaceHeritage",
      expression: null,
      typeArguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSTypePredicate(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypePredicate",
      parameterName: null,
      asserts: deserializeBool(pos + 32),
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.parameterName = deserializeTSTypePredicateName(pos + 8);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
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
    previousParent = parent,
    body = deserializeOptionTSModuleDeclarationBody(pos + 64);
  if (body === null) {
    node = parent = {
      type: "TSModuleDeclaration",
      id: null,
      kind,
      declare,
      global: false,
      start,
      end,
      range: [start, end],
      parent,
    };
    node.id = deserializeTSModuleDeclarationName(pos + 8);
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
    let id = deserializeTSModuleDeclarationName(pos + 8);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSModuleDeclaration",
      id: null,
      body: null,
      kind: null,
      declare: deserializeBool(pos + 76),
      global: null,
      start,
      end,
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
    start: (keywordStart = deserializeU32(pos + 8)),
    end: (keywordEnd = deserializeU32(pos + 12)),
    range: [keywordStart, keywordEnd],
    parent,
  };
  node.body = deserializeTSModuleBlock(pos + 16);
  node.kind = "global";
  node.global = true;
  parent = previousParent;
  return node;
}

function deserializeTSModuleBlock(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSModuleBlock",
      body: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    body = deserializeVecDirective(pos + 8);
  body.push(...deserializeVecStatement(pos + 32));
  node.body = body;
  parent = previousParent;
  return node;
}

function deserializeTSTypeLiteral(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeLiteral",
      members: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.members = deserializeVecTSSignature(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSInferType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSInferType",
      typeParameter: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeParameter = deserializeBoxTSTypeParameter(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSTypeQuery(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeQuery",
      exprName: null,
      typeArguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.exprName = deserializeTSTypeQueryExprName(pos + 8);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 24);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSImportType",
      source: null,
      options: null,
      qualifier: null,
      typeArguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.source = deserializeStringLiteral(pos + 8);
  node.options = deserializeOptionBoxObjectExpression(pos + 56);
  node.qualifier = deserializeOptionTSImportTypeQualifier(pos + 64);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 80);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.left = deserializeTSImportTypeQualifier(pos + 8);
  node.right = deserializeIdentifierName(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSFunctionType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSFunctionType",
      typeParameters: null,
      params: null,
      returnType: null,
      start,
      end,
      range: [start, end],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 24),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 16);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = params;
  node.returnType = deserializeBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSConstructorType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSConstructorType",
      abstract: deserializeBool(pos + 36),
      typeParameters: null,
      params: null,
      returnType: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 8);
  node.params = deserializeBoxFormalParameters(pos + 16);
  node.returnType = deserializeBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSMappedType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSMappedType",
      key: null,
      constraint: null,
      nameType: null,
      typeAnnotation: null,
      optional: null,
      readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 53),
      start,
      end,
      range: [start, end],
      parent,
    }),
    typeParameter = deserializeBoxTSTypeParameter(pos + 8),
    key = typeParameter.name;
  key.parent = parent;
  let { constraint } = typeParameter;
  constraint !== null && (constraint.parent = parent);
  let optional = deserializeOptionTSMappedTypeModifierOperator(pos + 52);
  optional === null && (optional = false);
  node.key = key;
  node.constraint = constraint;
  node.nameType = deserializeOptionTSType(pos + 16);
  node.typeAnnotation = deserializeOptionTSType(pos + 32);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTemplateLiteralType",
      quasis: null,
      types: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.quasis = deserializeVecTemplateElement(pos + 8);
  node.types = deserializeVecTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSAsExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSAsExpression",
      expression: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  node.typeAnnotation = deserializeTSType(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSSatisfiesExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSSatisfiesExpression",
      expression: null,
      typeAnnotation: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  node.typeAnnotation = deserializeTSType(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAssertion(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSTypeAssertion",
      typeAnnotation: null,
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
  node.expression = deserializeExpression(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSImportEqualsDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSImportEqualsDeclaration",
      id: null,
      moduleReference: null,
      importKind: deserializeImportOrExportKind(pos + 56),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 8);
  node.moduleReference = deserializeTSModuleReference(pos + 40);
  parent = previousParent;
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSExternalModuleReference",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeStringLiteral(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSNonNullExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSNonNullExpression",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeDecorator(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "Decorator",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSExportAssignment(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSExportAssignment",
      expression: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSNamespaceExportDeclaration",
      id: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.id = deserializeIdentifierName(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeTSInstantiationExpression(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSInstantiationExpression",
      expression: null,
      typeArguments: null,
      start,
      end,
      range: [start, end],
      parent,
    });
  node.expression = deserializeExpression(pos + 8);
  node.typeArguments = deserializeBoxTSTypeParameterInstantiation(pos + 24);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSJSDocNullableType",
      typeAnnotation: null,
      postfix: deserializeBool(pos + 24),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeJSDocNonNullableType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4),
    previousParent = parent,
    node = (parent = {
      type: "TSJSDocNonNullableType",
      typeAnnotation: null,
      postfix: deserializeBool(pos + 24),
      start,
      end,
      range: [start, end],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 8);
  parent = previousParent;
  return node;
}

function deserializeJSDocUnknownType(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type: "TSJSDocUnknownType",
    start,
    end,
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
    start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    type,
    value: sourceText.slice(start + 2, end - (type === "Line" ? 0 : 2)),
    start,
    end,
    range: [start, end],
  };
}

function deserializeNameSpan(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
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
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
  return {
    moduleRequest: deserializeNameSpan(pos + 8),
    entries: deserializeVecImportEntry(pos + 32),
    start,
    end,
    range: [start, end],
  };
}

function deserializeStaticExport(pos) {
  let start = deserializeU32(pos),
    end = deserializeU32(pos + 4);
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

function deserializeOptionTSAccessibility(pos) {
  if (uint8[pos] === 3) return null;
  return deserializeTSAccessibility(pos);
}

function deserializeVecTSClassImplements(pos) {
  let arr = [],
    pos32 = pos >> 2;
  pos = uint32[pos32];
  let endPos = pos + uint32[pos32 + 2] * 32;
  for (; pos !== endPos; ) {
    arr.push(deserializeTSClassImplements(pos));
    pos += 32;
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
