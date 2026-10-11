// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8,
  int32,
  float64,
  sourceText,
  sourceTextLatin,
  sourceStartPos = 0,
  firstNonAsciiPos = 0;

const { fromCharCode } = String,
  { utf8Slice, latin1Slice } = Buffer.prototype,
  stringDecodeArrays = Array(65).fill(null);
for (let i = 0; i <= 64; i++) stringDecodeArrays[i] = Array(i).fill(0);

export function deserialize(buffer, sourceText, sourceStartPos, sourceByteLen) {
  return deserializeWith(
    buffer,
    sourceText,
    sourceStartPos,
    sourceByteLen,
    deserializeRawTransferData,
  );
}

function deserializeWith(buffer, sourceTextInput, sourceStartPosInput, sourceByteLen, deserialize) {
  uint8 = buffer;
  int32 = buffer.int32;
  float64 = buffer.float64;
  sourceText = sourceTextInput;
  sourceStartPos = sourceStartPosInput;
  // Find first non-ASCII byte in source region.
  // `sourceText.substr()` can be used for strings which are within source text and ending before
  // this position, since byte offsets equal char offsets in the all-ASCII prefix.
  // Also decode source text as Latin-1 (or reuse `sourceText` if it's all ASCII).
  if (sourceText.length === sourceByteLen) {
    firstNonAsciiPos = sourceStartPos + sourceByteLen;
    sourceTextLatin = sourceText;
  } else {
    let i = sourceStartPos,
      sourceEndPos = sourceStartPos + sourceByteLen;
    for (; i < sourceEndPos && uint8[i] < 128; i++);
    firstNonAsciiPos = i;
    sourceTextLatin = latin1Slice.call(uint8, sourceStartPos, sourceEndPos);
  }
  let data = deserialize(int32[536870890]);
  resetBuffer();
  return data;
}

export function resetBuffer() {
  // Clear buffer and source text strings to allow them to be garbage collected
  uint8 = int32 = float64 = sourceText = sourceTextLatin = void 0;
}

// Append elements of `arr2` to `arr`, and return the combined array.
// Not `arr.push(...arr2)`, which throws a stack overflow error if `arr2` is very long
// (e.g. a file with 150k top-level statements).
// If `arr` is empty (e.g. no directives), return `arr2` instead of copying it.
function appendArray(arr, arr2) {
  if (arr.length === 0) return arr2;
  for (let i = 0; i < arr2.length; i++) arr.push(arr2[i]);
  return arr;
}

function deserializeProgram(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    program = {
      type: "Program",
      body: null,
      sourceType: deserializeModuleKind(pos + 137),
      hashbang: null,
      start,
      end,
      range: [start, end],
    };
  program.hashbang = deserializeOptionHashbang(pos + 56);
  program.body = appendArray(deserializeVecDirective(pos + 88), deserializeVecStatement(pos + 112));
  return program;
}

function deserializeExpression(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Expression`);
  }
}

function deserializeIdentifierName(pos) {
  let start, end;
  return {
    type: "Identifier",
    name: deserializeStr(pos + 16),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeIdentifierReference(pos) {
  let start, end;
  return {
    type: "Identifier",
    name: deserializeStr(pos + 16),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeBindingIdentifier(pos) {
  let start, end;
  return {
    type: "Identifier",
    name: deserializeStr(pos + 16),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeLabelIdentifier(pos) {
  let start, end;
  return {
    type: "Identifier",
    name: deserializeStr(pos + 16),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeThisExpression(pos) {
  let start, end;
  return {
    type: "ThisExpression",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeArrayExpression(pos) {
  let start,
    end,
    node = {
      type: "ArrayExpression",
      elements: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.elements = deserializeVecArrayExpressionElement(pos + 16);
  return node;
}

function deserializeArrayExpressionElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeSpreadElement(int32[(pos + 8) >> 2]);
    case 65:
      return deserializeElision(int32[(pos + 8) >> 2]);
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
    node = {
      type: "ObjectExpression",
      properties: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.properties = deserializeVecObjectPropertyKind(pos + 16);
  return node;
}

function deserializeObjectPropertyKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeObjectProperty(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeSpreadElement(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ObjectPropertyKind`);
  }
}

function deserializeObjectProperty(pos) {
  let start,
    end,
    node = {
      type: "Property",
      kind: deserializePropertyKind(pos + 12),
      key: null,
      value: null,
      method: uint8[pos + 13] === 1,
      shorthand: uint8[pos + 14] === 1,
      computed: uint8[pos + 15] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeExpression(pos + 32);
  return node;
}

function deserializePropertyKey(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeIdentifierName(int32[(pos + 8) >> 2]);
    case 65:
      return deserializePrivateIdentifier(int32[(pos + 8) >> 2]);
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
    node = {
      type: "TemplateLiteral",
      quasis: null,
      expressions: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.expressions = deserializeVecExpression(pos + 40);
  return node;
}

function deserializeTaggedTemplateExpression(pos) {
  let start,
    end,
    node = {
      type: "TaggedTemplateExpression",
      tag: null,
      quasi: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.tag = deserializeExpression(pos + 16);
  node.quasi = deserializeTemplateLiteral(pos + 40);
  return node;
}

function deserializeTemplateElement(pos) {
  let tail = uint8[pos + 12] === 1,
    start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    value = deserializeTemplateElementValue(pos + 16);
  value.cooked !== null
    && uint8[pos + 13] === 1
    && (value.cooked = value.cooked.replace(/\uFFFD(.{4})/g, (_, hex) =>
      String.fromCodePoint(parseInt(hex, 16)),
    ));
  return {
    type: "TemplateElement",
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
  let start,
    end,
    node = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: uint8[pos + 12] === 1,
      computed: true,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeExpression(pos + 32);
  return node;
}

function deserializeStaticMemberExpression(pos) {
  let start,
    end,
    node = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: uint8[pos + 12] === 1,
      computed: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeIdentifierName(pos + 32);
  return node;
}

function deserializePrivateFieldExpression(pos) {
  let start,
    end,
    node = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: uint8[pos + 12] === 1,
      computed: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.object = deserializeExpression(pos + 16);
  node.property = deserializePrivateIdentifier(pos + 32);
  return node;
}

function deserializeCallExpression(pos) {
  let start,
    end,
    node = {
      type: "CallExpression",
      callee: null,
      arguments: null,
      optional: uint8[pos + 12] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.callee = deserializeExpression(pos + 16);
  node.arguments = deserializeVecArgument(pos + 40);
  return node;
}

function deserializeNewExpression(pos) {
  let start,
    end,
    node = {
      type: "NewExpression",
      callee: null,
      arguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.callee = deserializeExpression(pos + 16);
  node.arguments = deserializeVecArgument(pos + 40);
  return node;
}

function deserializeImportMeta(pos) {
  let start,
    end,
    node = {
      type: "MetaProperty",
      meta: null,
      property: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    importStart = int32[pos >> 2],
    importEnd = int32[(pos + 4) >> 2] === 0 ? 0 : importStart + 6,
    importIdent = {
      type: "Identifier",
      name: "import",
      start: importStart,
      end: importEnd,
      range: [importStart, importEnd],
    },
    metaEnd = int32[(pos + 4) >> 2],
    metaStart = metaEnd === 0 ? 0 : metaEnd - 4,
    metaIdent = {
      type: "Identifier",
      name: "meta",
      start: metaStart,
      end: metaEnd,
      range: [metaStart, metaEnd],
    };
  node.meta = importIdent;
  node.property = metaIdent;
  return node;
}

function deserializeNewTarget(pos) {
  let start,
    end,
    node = {
      type: "MetaProperty",
      meta: null,
      property: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    newStart = int32[pos >> 2],
    newEnd = int32[(pos + 4) >> 2] === 0 ? 0 : newStart + 3,
    newIdent = {
      type: "Identifier",
      name: "new",
      start: newStart,
      end: newEnd,
      range: [newStart, newEnd],
    },
    targetEnd = int32[(pos + 4) >> 2],
    targetStart = targetEnd === 0 ? 0 : targetEnd - 6,
    targetIdent = {
      type: "Identifier",
      name: "target",
      start: targetStart,
      end: targetEnd,
      range: [targetStart, targetEnd],
    };
  node.meta = newIdent;
  node.property = targetIdent;
  return node;
}

function deserializeSpreadElement(pos) {
  let start,
    end,
    node = {
      type: "SpreadElement",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeExpression(pos + 16);
  return node;
}

function deserializeArgument(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeSpreadElement(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Argument`);
  }
}

function deserializeUpdateExpression(pos) {
  let start,
    end,
    node = {
      type: "UpdateExpression",
      operator: deserializeUpdateOperator(pos + 12),
      prefix: uint8[pos + 13] === 1,
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeSimpleAssignmentTarget(pos + 16);
  return node;
}

function deserializeUnaryExpression(pos) {
  let start,
    end,
    node = {
      type: "UnaryExpression",
      operator: deserializeUnaryOperator(pos + 12),
      argument: null,
      prefix: true,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeExpression(pos + 16);
  return node;
}

function deserializeBinaryExpression(pos) {
  let start,
    end,
    node = {
      type: "BinaryExpression",
      left: null,
      operator: deserializeBinaryOperator(pos + 12),
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  return node;
}

function deserializePrivateInExpression(pos) {
  let start,
    end,
    node = {
      type: "BinaryExpression",
      left: null,
      operator: "in",
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializePrivateIdentifier(pos + 16);
  node.right = deserializeExpression(pos + 48);
  return node;
}

function deserializeLogicalExpression(pos) {
  let start,
    end,
    node = {
      type: "LogicalExpression",
      left: null,
      operator: deserializeLogicalOperator(pos + 12),
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  return node;
}

function deserializeConditionalExpression(pos) {
  let start,
    end,
    node = {
      type: "ConditionalExpression",
      test: null,
      consequent: null,
      alternate: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeExpression(pos + 32);
  node.alternate = deserializeExpression(pos + 48);
  return node;
}

function deserializeAssignmentExpression(pos) {
  let start,
    end,
    node = {
      type: "AssignmentExpression",
      operator: deserializeAssignmentOperator(pos + 12),
      left: null,
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeAssignmentTarget(pos + 16);
  node.right = deserializeExpression(pos + 32);
  return node;
}

function deserializeAssignmentTarget(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeArrayAssignmentTarget(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeObjectAssignmentTarget(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTarget`);
  }
}

function deserializeSimpleAssignmentTarget(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for SimpleAssignmentTarget`);
  }
}

function deserializeArrayAssignmentTarget(pos) {
  let start,
    end,
    node = {
      type: "ArrayPattern",
      elements: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  return node;
}

function deserializeObjectAssignmentTarget(pos) {
  let start,
    end,
    node = {
      type: "ObjectPattern",
      properties: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    properties = deserializeVecAssignmentTargetProperty(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  return node;
}

function deserializeAssignmentTargetRest(pos) {
  let start,
    end,
    node = {
      type: "RestElement",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeAssignmentTarget(pos + 16);
  return node;
}

function deserializeAssignmentTargetMaybeDefault(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeArrayAssignmentTarget(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeObjectAssignmentTarget(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeAssignmentTargetWithDefault(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetMaybeDefault`);
  }
}

function deserializeAssignmentTargetWithDefault(pos) {
  let start,
    end,
    node = {
      type: "AssignmentPattern",
      left: null,
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeAssignmentTarget(pos + 16);
  node.right = deserializeExpression(pos + 32);
  return node;
}

function deserializeAssignmentTargetProperty(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeAssignmentTargetPropertyIdentifier(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeAssignmentTargetPropertyProperty(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetProperty`);
  }
}

function deserializeAssignmentTargetPropertyIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    node = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: true,
      computed: false,
      start,
      end,
      range: [start, end],
    },
    key = deserializeIdentifierReference(pos + 16),
    keyStart,
    keyEnd,
    value = {
      type: "Identifier",
      name: key.name,
      start: (keyStart = key.start),
      end: (keyEnd = key.end),
      range: [keyStart, keyEnd],
    },
    init = deserializeOptionExpression(pos + 48);
  init !== null
    && (value = {
      type: "AssignmentPattern",
      left: value,
      right: init,
      start,
      end,
      range: [start, end],
    });
  node.key = key;
  node.value = value;
  return node;
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let start,
    end,
    node = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: false,
      computed: uint8[pos + 12] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeAssignmentTargetMaybeDefault(pos + 32);
  return node;
}

function deserializeSequenceExpression(pos) {
  let start,
    end,
    node = {
      type: "SequenceExpression",
      expressions: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expressions = deserializeVecExpression(pos + 16);
  return node;
}

function deserializeSuper(pos) {
  let start, end;
  return {
    type: "Super",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeAwaitExpression(pos) {
  let start,
    end,
    node = {
      type: "AwaitExpression",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeExpression(pos + 16);
  return node;
}

function deserializeChainExpression(pos) {
  let start,
    end,
    node = {
      type: "ChainExpression",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeChainElement(pos + 16);
  return node;
}

function deserializeChainElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ChainElement`);
  }
}

function deserializeParenthesizedExpression(pos) {
  let node;
  {
    let start, end;
    node = {
      type: "ParenthesizedExpression",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
    node.expression = deserializeExpression(pos + 16);
  }
  return node;
}

function deserializeStatement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBlockStatement(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeBreakStatement(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeContinueStatement(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeDebuggerStatement(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeDoWhileStatement(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeEmptyStatement(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeExpressionStatement(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeForInStatement(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeForOfStatement(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeForStatement(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeIfStatement(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeLabeledStatement(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeReturnStatement(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeSwitchStatement(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeThrowStatement(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeTryStatement(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeWhileStatement(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeWithStatement(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeVariableDeclaration(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSTypeAliasDeclaration(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSInterfaceDeclaration(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSEnumDeclaration(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSExternalModuleDeclaration(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSNamespaceDeclaration(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeTSGlobalDeclaration(int32[(pos + 8) >> 2]);
    case 41:
      return deserializeTSImportEqualsDeclaration(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeImportDeclaration(int32[(pos + 8) >> 2]);
    case 65:
      return deserializeExportAllDeclaration(int32[(pos + 8) >> 2]);
    case 66:
      return deserializeExportDefaultDeclaration(int32[(pos + 8) >> 2]);
    case 67:
      return deserializeExportDeclaration(int32[(pos + 8) >> 2]);
    case 68:
      return deserializeExportNamedDeclaration(int32[(pos + 8) >> 2]);
    case 69:
      return deserializeExportFromDeclaration(int32[(pos + 8) >> 2]);
    case 70:
      return deserializeTSExportAssignment(int32[(pos + 8) >> 2]);
    case 71:
      return deserializeTSNamespaceExportDeclaration(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Statement`);
  }
}

function deserializeDirective(pos) {
  let start,
    end,
    node = {
      type: "ExpressionStatement",
      expression: null,
      directive: deserializeStr(pos + 64),
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeStringLiteral(pos + 16);
  return node;
}

function deserializeHashbang(pos) {
  let start, end;
  return {
    type: "Hashbang",
    value: deserializeStr(pos + 16),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeBlockStatement(pos) {
  let start,
    end,
    node = {
      type: "BlockStatement",
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.body = deserializeVecStatement(pos + 16);
  return node;
}

function deserializeDeclaration(pos) {
  switch (uint8[pos]) {
    case 32:
      return deserializeVariableDeclaration(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSTypeAliasDeclaration(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSInterfaceDeclaration(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSEnumDeclaration(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSExternalModuleDeclaration(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSNamespaceDeclaration(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeTSGlobalDeclaration(int32[(pos + 8) >> 2]);
    case 41:
      return deserializeTSImportEqualsDeclaration(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Declaration`);
  }
}

function deserializeVariableDeclaration(pos) {
  let start,
    end,
    node = {
      type: "VariableDeclaration",
      kind: deserializeVariableDeclarationKind(pos + 12),
      declarations: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.declarations = deserializeVecVariableDeclarator(pos + 16);
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
    node = {
      type: "VariableDeclarator",
      id: null,
      init: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeBindingPattern(pos + 16);
  node.init = deserializeOptionExpression(pos + 40);
  return node;
}

function deserializeEmptyStatement(pos) {
  let start, end;
  return {
    type: "EmptyStatement",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeExpressionStatement(pos) {
  let start,
    end,
    node = {
      type: "ExpressionStatement",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  return node;
}

function deserializeIfStatement(pos) {
  let start,
    end,
    node = {
      type: "IfStatement",
      test: null,
      consequent: null,
      alternate: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeStatement(pos + 32);
  node.alternate = deserializeOptionStatement(pos + 48);
  return node;
}

function deserializeDoWhileStatement(pos) {
  let start,
    end,
    node = {
      type: "DoWhileStatement",
      body: null,
      test: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.body = deserializeStatement(pos + 16);
  node.test = deserializeExpression(pos + 32);
  return node;
}

function deserializeWhileStatement(pos) {
  let start,
    end,
    node = {
      type: "WhileStatement",
      test: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.test = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  return node;
}

function deserializeForStatement(pos) {
  let start,
    end,
    node = {
      type: "ForStatement",
      init: null,
      test: null,
      update: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.init = deserializeOptionForStatementInit(pos + 16);
  node.test = deserializeOptionExpression(pos + 32);
  node.update = deserializeOptionExpression(pos + 48);
  node.body = deserializeStatement(pos + 64);
  return node;
}

function deserializeForStatementInit(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeVariableDeclaration(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ForStatementInit`);
  }
}

function deserializeForInStatement(pos) {
  let start,
    end,
    node = {
      type: "ForInStatement",
      left: null,
      right: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  return node;
}

function deserializeForStatementLeft(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeArrayAssignmentTarget(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeObjectAssignmentTarget(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeVariableDeclaration(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ForStatementLeft`);
  }
}

function deserializeForOfStatement(pos) {
  let start,
    end,
    node = {
      type: "ForOfStatement",
      await: uint8[pos + 64] === 1,
      left: null,
      right: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  return node;
}

function deserializeContinueStatement(pos) {
  let start,
    end,
    node = {
      type: "ContinueStatement",
      label: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  return node;
}

function deserializeBreakStatement(pos) {
  let start,
    end,
    node = {
      type: "BreakStatement",
      label: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  return node;
}

function deserializeReturnStatement(pos) {
  let start,
    end,
    node = {
      type: "ReturnStatement",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeOptionExpression(pos + 16);
  return node;
}

function deserializeWithStatement(pos) {
  let start,
    end,
    node = {
      type: "WithStatement",
      object: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.object = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  return node;
}

function deserializeSwitchStatement(pos) {
  let start,
    end,
    node = {
      type: "SwitchStatement",
      discriminant: null,
      cases: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.discriminant = deserializeExpression(pos + 16);
  node.cases = deserializeVecSwitchCase(pos + 32);
  return node;
}

function deserializeSwitchCase(pos) {
  let start,
    end,
    node = {
      type: "SwitchCase",
      test: null,
      consequent: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.test = deserializeOptionExpression(pos + 16);
  node.consequent = deserializeVecStatement(pos + 32);
  return node;
}

function deserializeLabeledStatement(pos) {
  let start,
    end,
    node = {
      type: "LabeledStatement",
      label: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.label = deserializeLabelIdentifier(pos + 16);
  node.body = deserializeStatement(pos + 48);
  return node;
}

function deserializeThrowStatement(pos) {
  let start,
    end,
    node = {
      type: "ThrowStatement",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeExpression(pos + 16);
  return node;
}

function deserializeTryStatement(pos) {
  let start,
    end,
    node = {
      type: "TryStatement",
      block: null,
      handler: null,
      finalizer: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.block = deserializeBlockStatement(int32[(pos + 16) >> 2]);
  node.handler = deserializeOptionBoxCatchClause(pos + 24);
  node.finalizer = deserializeOptionBoxBlockStatement(pos + 32);
  return node;
}

function deserializeCatchClause(pos) {
  let start,
    end,
    node = {
      type: "CatchClause",
      param: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.param = deserializeOptionCatchParameter(pos + 16);
  node.body = deserializeBlockStatement(int32[(pos + 56) >> 2]);
  return node;
}

function deserializeCatchParameter(pos) {
  return deserializeBindingPattern(pos + 16);
}

function deserializeDebuggerStatement(pos) {
  let start, end;
  return {
    type: "DebuggerStatement",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeBindingPattern(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBindingIdentifier(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeObjectPattern(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeArrayPattern(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeAssignmentPattern(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for BindingPattern`);
  }
}

function deserializeAssignmentPattern(pos) {
  let start,
    end,
    node = {
      type: "AssignmentPattern",
      left: null,
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeBindingPattern(pos + 16);
  node.right = deserializeExpression(pos + 32);
  return node;
}

function deserializeObjectPattern(pos) {
  let start,
    end,
    node = {
      type: "ObjectPattern",
      properties: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    properties = deserializeVecBindingProperty(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  return node;
}

function deserializeBindingProperty(pos) {
  let start,
    end,
    node = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: uint8[pos + 12] === 1,
      computed: uint8[pos + 13] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeBindingPattern(pos + 32);
  return node;
}

function deserializeArrayPattern(pos) {
  let start,
    end,
    node = {
      type: "ArrayPattern",
      elements: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    elements = deserializeVecOptionBindingPattern(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  return node;
}

function deserializeBindingRestElement(pos) {
  let start,
    end,
    node = {
      type: "RestElement",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeBindingPattern(pos + 16);
  return node;
}

function deserializeFunction(pos) {
  let start,
    end,
    node = {
      type: deserializeFunctionType(pos + 88),
      id: null,
      generator: uint8[pos + 89] === 1,
      async: uint8[pos + 90] === 1,
      params: null,
      body: null,
      expression: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    params = deserializeFormalParameters(int32[(pos + 64) >> 2]);
  node.id = deserializeOptionBindingIdentifier(pos + 16);
  node.params = params;
  node.body = deserializeOptionBoxFunctionBody(pos + 80);
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
      rest = {
        type: "RestElement",
        argument: null,
        start: (start = int32[(pos + 40) >> 2]),
        end: (end = int32[(pos + 44) >> 2]),
        range: [start, end],
      };
    rest.argument = deserializeBindingPattern(pos + 56);
    params.push(rest);
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param,
    initializerFieldPos32 = (pos >> 2) + 16;
  if (int32[initializerFieldPos32] !== 0 && int32[initializerFieldPos32 + 1] !== 0) {
    let start, end;
    param = {
      type: "AssignmentPattern",
      left: null,
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
    param.left = deserializeBindingPattern(pos + 40);
    param.right = deserializeOptionBoxExpression(pos + 64);
  } else param = deserializeBindingPattern(pos + 40);
  return param;
}

function deserializeFunctionBody(pos) {
  let start,
    end,
    node = {
      type: "BlockStatement",
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    body = deserializeVecDirective(pos + 16);
  body = appendArray(body, deserializeVecStatement(pos + 40));
  node.body = body;
  return node;
}

function deserializeArrowFunctionBody(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeFunctionBody(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ArrowFunctionBody`);
  }
}

function deserializeArrowFunctionExpression(pos) {
  let start,
    end,
    node = {
      type: "ArrowFunctionExpression",
      expression: uint8[pos + 40] !== 64,
      async: uint8[pos + 56] === 1,
      params: null,
      body: null,
      id: null,
      generator: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.params = deserializeFormalParameters(int32[(pos + 24) >> 2]);
  node.body = deserializeArrowFunctionBody(pos + 40);
  return node;
}

function deserializeYieldExpression(pos) {
  let start,
    end,
    node = {
      type: "YieldExpression",
      delegate: uint8[pos + 12] === 1,
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeOptionExpression(pos + 16);
  return node;
}

function deserializeClass(pos) {
  let start,
    end,
    node = {
      type: deserializeClassType(pos + 136),
      decorators: null,
      id: null,
      superClass: null,
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    superClass = deserializeOptionExpression(pos + 80);
  node.decorators = deserializeVecDecorator(pos + 16);
  node.id = deserializeOptionBindingIdentifier(pos + 40);
  node.superClass = superClass;
  node.body = deserializeClassBody(int32[(pos + 128) >> 2]);
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
    node = {
      type: "ClassBody",
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.body = deserializeVecClassElement(pos + 16);
  return node;
}

function deserializeClassElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeStaticBlock(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeMethodDefinition(int32[(pos + 8) >> 2]);
    case 2:
      return deserializePropertyDefinition(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeAccessorProperty(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSIndexSignature(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ClassElement`);
  }
}

function deserializeMethodDefinition(pos) {
  let start,
    end,
    node = {
      type: deserializeMethodDefinitionType(pos + 12),
      decorators: null,
      key: null,
      value: null,
      kind: deserializeMethodDefinitionKind(pos + 13),
      computed: uint8[pos + 14] === 1,
      static: uint8[pos + 15] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.value = deserializeFunction(int32[(pos + 56) >> 2]);
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
    node = {
      type: deserializePropertyDefinitionType(pos + 12),
      decorators: null,
      key: null,
      value: null,
      computed: uint8[pos + 13] === 1,
      static: uint8[pos + 14] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.value = deserializeOptionExpression(pos + 64);
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
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeStaticBlock(pos) {
  let start,
    end,
    node = {
      type: "StaticBlock",
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.body = deserializeVecStatement(pos + 16);
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
    node = {
      type: deserializeAccessorPropertyType(pos + 12),
      decorators: null,
      key: null,
      value: null,
      computed: uint8[pos + 13] === 1,
      static: uint8[pos + 14] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.value = deserializeOptionExpression(pos + 64);
  return node;
}

function deserializeImportExpression(pos) {
  let start,
    end,
    node = {
      type: "ImportExpression",
      source: null,
      options: null,
      phase: deserializeOptionImportPhase(pos + 12),
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.source = deserializeExpression(pos + 16);
  node.options = deserializeOptionExpression(pos + 32);
  return node;
}

function deserializeImportDeclaration(pos) {
  let start,
    end,
    node = {
      type: "ImportDeclaration",
      specifiers: null,
      source: null,
      phase: deserializeOptionImportPhase(pos + 12),
      attributes: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 16);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 88);
  node.specifiers = specifiers;
  node.source = deserializeStringLiteral(pos + 40);
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
      return deserializeImportSpecifier(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeImportDefaultSpecifier(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeImportNamespaceSpecifier(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportDeclarationSpecifier`);
  }
}

function deserializeImportSpecifier(pos) {
  let start,
    end,
    node = {
      type: "ImportSpecifier",
      imported: null,
      local: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.imported = deserializeModuleExportName(pos + 16);
  node.local = deserializeBindingIdentifier(pos + 72);
  return node;
}

function deserializeImportDefaultSpecifier(pos) {
  let start,
    end,
    node = {
      type: "ImportDefaultSpecifier",
      local: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.local = deserializeBindingIdentifier(pos + 16);
  return node;
}

function deserializeImportNamespaceSpecifier(pos) {
  let start,
    end,
    node = {
      type: "ImportNamespaceSpecifier",
      local: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.local = deserializeBindingIdentifier(pos + 16);
  return node;
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 16) };
}

function deserializeImportAttribute(pos) {
  let start,
    end,
    node = {
      type: "ImportAttribute",
      key: null,
      value: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.key = deserializeImportAttributeKey(pos + 16);
  node.value = deserializeStringLiteral(pos + 72);
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

function deserializeExportDeclaration(pos) {
  let start,
    end,
    node = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: [],
      source: null,
      attributes: [],
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.declaration = deserializeDeclaration(pos + 16);
  return node;
}

function deserializeExportNamedDeclaration(pos) {
  let start,
    end,
    node = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      attributes: [],
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.specifiers = deserializeVecExportSpecifier(pos + 16);
  return node;
}

function deserializeExportFromDeclaration(pos) {
  let start,
    end,
    node = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      attributes: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    withClause = deserializeOptionBoxWithClause(pos + 88);
  node.specifiers = deserializeVecExportSpecifier(pos + 16);
  node.source = deserializeStringLiteral(pos + 40);
  node.attributes = withClause === null ? [] : withClause.attributes;
  return node;
}

function deserializeExportDefaultDeclaration(pos) {
  let start,
    end,
    node = {
      type: "ExportDefaultDeclaration",
      declaration: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.declaration = deserializeExportDefaultDeclarationKind(pos + 16);
  return node;
}

function deserializeExportAllDeclaration(pos) {
  let start,
    end,
    node = {
      type: "ExportAllDeclaration",
      exported: null,
      source: null,
      attributes: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    withClause = deserializeOptionBoxWithClause(pos + 120);
  node.exported = deserializeOptionModuleExportName(pos + 16);
  node.source = deserializeStringLiteral(pos + 72);
  node.attributes = withClause === null ? [] : withClause.attributes;
  return node;
}

function deserializeExportSpecifier(pos) {
  let start,
    end,
    node = {
      type: "ExportSpecifier",
      local: null,
      exported: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.local = deserializeModuleExportName(pos + 16);
  node.exported = deserializeModuleExportName(pos + 72);
  return node;
}

function deserializeExportDefaultDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 65:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 66:
      return deserializeTSInterfaceDeclaration(int32[(pos + 8) >> 2]);
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
    node = {
      type: "V8IntrinsicExpression",
      name: null,
      arguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.name = deserializeIdentifierName(pos + 16);
  node.arguments = deserializeVecArgument(pos + 48);
  return node;
}

function deserializeBooleanLiteral(pos) {
  let value = uint8[pos + 12] === 1,
    start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    node = {
      type: "Literal",
      value,
      raw: null,
      start,
      end,
      range: [start, end],
    };
  node.raw = start === 0 && end === 0 ? null : value + "";
  return node;
}

function deserializeNullLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2];
  return {
    type: "Literal",
    value: null,
    raw: start === 0 && end === 0 ? null : "null",
    start,
    end,
    range: [start, end],
  };
}

function deserializeNumericLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2];
  return {
    type: "Literal",
    value: float64[(pos + 32) >> 3],
    raw:
      int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
        ? null
        : sourceText.slice(start, end),
    start,
    end,
    range: [start, end],
  };
}

function deserializeStringLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    node = {
      type: "Literal",
      value: null,
      raw:
        int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
          ? null
          : sourceText.slice(start, end),
      start,
      end,
      range: [start, end],
    },
    value = deserializeStr(pos + 16);
  uint8[pos + 12] === 1
    && (value = value.replace(/\uFFFD(.{4})/g, (_, hex) =>
      String.fromCodePoint(parseInt(hex, 16)),
    ));
  node.value = value;
  return node;
}

function deserializeBigIntLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
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
  };
}

function deserializeRegExpLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    node = {
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
    },
    regex = deserializeRegExp(pos + 16),
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
  let flagBits = uint8[pos],
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
    node = {
      type: "JSXElement",
      openingElement: null,
      children: null,
      closingElement: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    closingElement = deserializeOptionBoxJSXClosingElement(pos + 48),
    openingElement = deserializeJSXOpeningElement(int32[(pos + 16) >> 2]);
  closingElement === null && (openingElement.selfClosing = true);
  node.openingElement = openingElement;
  node.children = deserializeVecJSXChild(pos + 24);
  node.closingElement = closingElement;
  return node;
}

function deserializeJSXOpeningElement(pos) {
  let start,
    end,
    node = {
      type: "JSXOpeningElement",
      name: null,
      attributes: null,
      selfClosing: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.name = deserializeJSXElementName(pos + 16);
  node.attributes = deserializeVecJSXAttributeItem(pos + 40);
  return node;
}

function deserializeJSXClosingElement(pos) {
  let start,
    end,
    node = {
      type: "JSXClosingElement",
      name: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.name = deserializeJSXElementName(pos + 16);
  return node;
}

function deserializeJSXFragment(pos) {
  let start,
    end,
    node = {
      type: "JSXFragment",
      openingFragment: null,
      children: null,
      closingFragment: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.openingFragment = deserializeJSXOpeningFragment(pos + 16);
  node.children = deserializeVecJSXChild(pos + 32);
  node.closingFragment = deserializeJSXClosingFragment(pos + 56);
  return node;
}

function deserializeJSXOpeningFragment(pos) {
  let start, end;
  return {
    type: "JSXOpeningFragment",
    attributes: [],
    selfClosing: false,
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeJSXClosingFragment(pos) {
  let start, end;
  return {
    type: "JSXClosingFragment",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeJSXElementName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXIdentifier(int32[(pos + 8) >> 2]);
    case 1:
      let ident = deserializeIdentifierReference(int32[(pos + 8) >> 2]);
      return {
        type: "JSXIdentifier",
        name: ident.name,
        start: ident.start,
        end: ident.end,
        range: ident.range,
      };
    case 2:
      return deserializeJSXNamespacedName(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeJSXMemberExpression(int32[(pos + 8) >> 2]);
    case 4:
      let thisExpr = deserializeThisExpression(int32[(pos + 8) >> 2]);
      return {
        type: "JSXIdentifier",
        name: "this",
        start: thisExpr.start,
        end: thisExpr.end,
        range: thisExpr.range,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  let start,
    end,
    node = {
      type: "JSXNamespacedName",
      namespace: null,
      name: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.namespace = deserializeJSXIdentifier(pos + 16);
  node.name = deserializeJSXIdentifier(pos + 48);
  return node;
}

function deserializeJSXMemberExpression(pos) {
  let start,
    end,
    node = {
      type: "JSXMemberExpression",
      object: null,
      property: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.object = deserializeJSXMemberExpressionObject(pos + 16);
  node.property = deserializeJSXIdentifier(pos + 32);
  return node;
}

function deserializeJSXMemberExpressionObject(pos) {
  switch (uint8[pos]) {
    case 0:
      let ident = deserializeIdentifierReference(int32[(pos + 8) >> 2]);
      return {
        type: "JSXIdentifier",
        name: ident.name,
        start: ident.start,
        end: ident.end,
        range: ident.range,
      };
    case 1:
      return deserializeJSXMemberExpression(int32[(pos + 8) >> 2]);
    case 2:
      let thisExpr = deserializeThisExpression(int32[(pos + 8) >> 2]);
      return {
        type: "JSXIdentifier",
        name: "this",
        start: thisExpr.start,
        end: thisExpr.end,
        range: thisExpr.range,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  let start,
    end,
    node = {
      type: "JSXExpressionContainer",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeJSXExpression(pos + 16);
  return node;
}

function deserializeJSXExpression(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeSuper(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeCallExpression(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeChainExpression(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeClass(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeFunction(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeImportExpression(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeNewExpression(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos + 8) >> 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos + 8) >> 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeImportMeta(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeNewTarget(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos + 8) >> 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos + 8) >> 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos + 8) >> 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos + 8) >> 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos + 8) >> 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos + 8) >> 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeJSXEmptyExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXExpression`);
  }
}

function deserializeJSXEmptyExpression(pos) {
  let start, end;
  return {
    type: "JSXEmptyExpression",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeJSXAttributeItem(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXAttribute(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeJSXSpreadAttribute(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeItem`);
  }
}

function deserializeJSXAttribute(pos) {
  let start,
    end,
    node = {
      type: "JSXAttribute",
      name: null,
      value: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.name = deserializeJSXAttributeName(pos + 16);
  node.value = deserializeOptionJSXAttributeValue(pos + 32);
  return node;
}

function deserializeJSXSpreadAttribute(pos) {
  let start,
    end,
    node = {
      type: "JSXSpreadAttribute",
      argument: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.argument = deserializeExpression(pos + 16);
  return node;
}

function deserializeJSXAttributeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXIdentifier(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeJSXNamespacedName(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeName`);
  }
}

function deserializeJSXAttributeValue(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeJSXExpressionContainer(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeValue`);
  }
}

function deserializeJSXIdentifier(pos) {
  let start, end;
  return {
    type: "JSXIdentifier",
    name: deserializeStr(pos + 16),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeJSXChild(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXText(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeJSXElement(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeJSXFragment(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeJSXExpressionContainer(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeJSXSpreadChild(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXChild`);
  }
}

function deserializeJSXSpreadChild(pos) {
  let start,
    end,
    node = {
      type: "JSXSpreadChild",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  return node;
}

function deserializeJSXText(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2];
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
  };
}

function deserializeTSThisParameter(pos) {
  let start,
    end,
    node = {
      type: "Identifier",
      decorators: [],
      name: "this",
      optional: false,
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  return node;
}

function deserializeTSEnumDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSEnumDeclaration",
      id: null,
      body: null,
      const: uint8[pos + 12] === 1,
      declare: uint8[pos + 13] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeBindingIdentifier(pos + 16);
  node.body = deserializeTSEnumBody(pos + 48);
  return node;
}

function deserializeTSEnumBody(pos) {
  let start,
    end,
    node = {
      type: "TSEnumBody",
      members: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.members = deserializeVecTSEnumMember(pos + 16);
  return node;
}

function deserializeTSEnumMember(pos) {
  let start,
    end,
    node = {
      type: "TSEnumMember",
      id: null,
      initializer: null,
      computed: uint8[pos + 16] > 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeTSEnumMemberName(pos + 16);
  node.initializer = deserializeOptionExpression(pos + 32);
  return node;
}

function deserializeTSEnumMemberName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSEnumMemberName`);
  }
}

function deserializeTSTypeAnnotation(pos) {
  let start,
    end,
    node = {
      type: "TSTypeAnnotation",
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
  return node;
}

function deserializeTSLiteralType(pos) {
  let start,
    end,
    node = {
      type: "TSLiteralType",
      literal: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.literal = deserializeTSLiteral(pos + 16);
  return node;
}

function deserializeTSLiteral(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeNumericLiteral(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeBigIntLiteral(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeStringLiteral(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTemplateLiteral(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeUnaryExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSLiteral`);
  }
}

function deserializeTSType(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSAnyKeyword(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSBigIntKeyword(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSBooleanKeyword(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSIntrinsicKeyword(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSNeverKeyword(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeTSNullKeyword(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTSNumberKeyword(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeTSObjectKeyword(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeTSStringKeyword(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeTSSymbolKeyword(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeTSThisType(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeTSUndefinedKeyword(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeTSUnknownKeyword(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeTSVoidKeyword(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeTSArrayType(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeTSConditionalType(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeTSConstructorType(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeTSFunctionType(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeTSImportType(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeTSIndexedAccessType(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeTSInferType(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeTSIntersectionType(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeTSLiteralType(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeTSMappedType(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeTSNamedTupleMember(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeTSTemplateLiteralType(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeTSTupleType(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeTSTypeLiteral(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeTSTypeOperator(int32[(pos + 8) >> 2]);
    case 30:
      return deserializeTSTypePredicate(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeTSTypeQuery(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeTSTypeReference(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeTSUnionType(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeTSParenthesizedType(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeJSDocNullableType(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeJSDocNonNullableType(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeJSDocUnknownType(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSType`);
  }
}

function deserializeTSConditionalType(pos) {
  let start,
    end,
    node = {
      type: "TSConditionalType",
      checkType: null,
      extendsType: null,
      trueType: null,
      falseType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.checkType = deserializeTSType(pos + 16);
  node.extendsType = deserializeTSType(pos + 32);
  node.trueType = deserializeTSType(pos + 48);
  node.falseType = deserializeTSType(pos + 64);
  return node;
}

function deserializeTSUnionType(pos) {
  let start,
    end,
    node = {
      type: "TSUnionType",
      types: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.types = deserializeVecTSType(pos + 16);
  return node;
}

function deserializeTSIntersectionType(pos) {
  let start,
    end,
    node = {
      type: "TSIntersectionType",
      types: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.types = deserializeVecTSType(pos + 16);
  return node;
}

function deserializeTSParenthesizedType(pos) {
  let node;
  {
    let start, end;
    node = {
      type: "TSParenthesizedType",
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
    node.typeAnnotation = deserializeTSType(pos + 16);
  }
  return node;
}

function deserializeTSTypeOperator(pos) {
  let start,
    end,
    node = {
      type: "TSTypeOperator",
      operator: deserializeTSTypeOperatorOperator(pos + 12),
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
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
    node = {
      type: "TSArrayType",
      elementType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.elementType = deserializeTSType(pos + 16);
  return node;
}

function deserializeTSIndexedAccessType(pos) {
  let start,
    end,
    node = {
      type: "TSIndexedAccessType",
      objectType: null,
      indexType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.objectType = deserializeTSType(pos + 16);
  node.indexType = deserializeTSType(pos + 32);
  return node;
}

function deserializeTSTupleType(pos) {
  let start,
    end,
    node = {
      type: "TSTupleType",
      elementTypes: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.elementTypes = deserializeVecTSTupleElement(pos + 16);
  return node;
}

function deserializeTSNamedTupleMember(pos) {
  let start,
    end,
    node = {
      type: "TSNamedTupleMember",
      label: null,
      elementType: null,
      optional: uint8[pos + 12] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.label = deserializeIdentifierName(pos + 16);
  node.elementType = deserializeTSTupleElement(pos + 48);
  return node;
}

function deserializeTSOptionalType(pos) {
  let start,
    end,
    node = {
      type: "TSOptionalType",
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
  return node;
}

function deserializeTSRestType(pos) {
  let start,
    end,
    node = {
      type: "TSRestType",
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
  return node;
}

function deserializeTSTupleElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSAnyKeyword(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSBigIntKeyword(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSBooleanKeyword(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSIntrinsicKeyword(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSNeverKeyword(int32[(pos + 8) >> 2]);
    case 5:
      return deserializeTSNullKeyword(int32[(pos + 8) >> 2]);
    case 6:
      return deserializeTSNumberKeyword(int32[(pos + 8) >> 2]);
    case 7:
      return deserializeTSObjectKeyword(int32[(pos + 8) >> 2]);
    case 8:
      return deserializeTSStringKeyword(int32[(pos + 8) >> 2]);
    case 9:
      return deserializeTSSymbolKeyword(int32[(pos + 8) >> 2]);
    case 10:
      return deserializeTSThisType(int32[(pos + 8) >> 2]);
    case 11:
      return deserializeTSUndefinedKeyword(int32[(pos + 8) >> 2]);
    case 12:
      return deserializeTSUnknownKeyword(int32[(pos + 8) >> 2]);
    case 13:
      return deserializeTSVoidKeyword(int32[(pos + 8) >> 2]);
    case 14:
      return deserializeTSArrayType(int32[(pos + 8) >> 2]);
    case 15:
      return deserializeTSConditionalType(int32[(pos + 8) >> 2]);
    case 16:
      return deserializeTSConstructorType(int32[(pos + 8) >> 2]);
    case 17:
      return deserializeTSFunctionType(int32[(pos + 8) >> 2]);
    case 18:
      return deserializeTSImportType(int32[(pos + 8) >> 2]);
    case 19:
      return deserializeTSIndexedAccessType(int32[(pos + 8) >> 2]);
    case 20:
      return deserializeTSInferType(int32[(pos + 8) >> 2]);
    case 21:
      return deserializeTSIntersectionType(int32[(pos + 8) >> 2]);
    case 22:
      return deserializeTSLiteralType(int32[(pos + 8) >> 2]);
    case 23:
      return deserializeTSMappedType(int32[(pos + 8) >> 2]);
    case 24:
      return deserializeTSNamedTupleMember(int32[(pos + 8) >> 2]);
    case 26:
      return deserializeTSTemplateLiteralType(int32[(pos + 8) >> 2]);
    case 27:
      return deserializeTSTupleType(int32[(pos + 8) >> 2]);
    case 28:
      return deserializeTSTypeLiteral(int32[(pos + 8) >> 2]);
    case 29:
      return deserializeTSTypeOperator(int32[(pos + 8) >> 2]);
    case 30:
      return deserializeTSTypePredicate(int32[(pos + 8) >> 2]);
    case 31:
      return deserializeTSTypeQuery(int32[(pos + 8) >> 2]);
    case 32:
      return deserializeTSTypeReference(int32[(pos + 8) >> 2]);
    case 33:
      return deserializeTSUnionType(int32[(pos + 8) >> 2]);
    case 34:
      return deserializeTSParenthesizedType(int32[(pos + 8) >> 2]);
    case 35:
      return deserializeJSDocNullableType(int32[(pos + 8) >> 2]);
    case 36:
      return deserializeJSDocNonNullableType(int32[(pos + 8) >> 2]);
    case 37:
      return deserializeJSDocUnknownType(int32[(pos + 8) >> 2]);
    case 64:
      return deserializeTSOptionalType(int32[(pos + 8) >> 2]);
    case 65:
      return deserializeTSRestType(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTupleElement`);
  }
}

function deserializeTSAnyKeyword(pos) {
  let start, end;
  return {
    type: "TSAnyKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSStringKeyword(pos) {
  let start, end;
  return {
    type: "TSStringKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSBooleanKeyword(pos) {
  let start, end;
  return {
    type: "TSBooleanKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSNumberKeyword(pos) {
  let start, end;
  return {
    type: "TSNumberKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSNeverKeyword(pos) {
  let start, end;
  return {
    type: "TSNeverKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  let start, end;
  return {
    type: "TSIntrinsicKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSUnknownKeyword(pos) {
  let start, end;
  return {
    type: "TSUnknownKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSNullKeyword(pos) {
  let start, end;
  return {
    type: "TSNullKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSUndefinedKeyword(pos) {
  let start, end;
  return {
    type: "TSUndefinedKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSVoidKeyword(pos) {
  let start, end;
  return {
    type: "TSVoidKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSSymbolKeyword(pos) {
  let start, end;
  return {
    type: "TSSymbolKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSThisType(pos) {
  let start, end;
  return {
    type: "TSThisType",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSObjectKeyword(pos) {
  let start, end;
  return {
    type: "TSObjectKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSBigIntKeyword(pos) {
  let start, end;
  return {
    type: "TSBigIntKeyword",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeTSTypeReference(pos) {
  let start,
    end,
    node = {
      type: "TSTypeReference",
      typeName: null,
      typeArguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeName = deserializeTSTypeName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  return node;
}

function deserializeTSTypeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSQualifiedName(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeName`);
  }
}

function deserializeTSQualifiedName(pos) {
  let start,
    end,
    node = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeTSTypeName(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  return node;
}

function deserializeTSTypeParameterInstantiation(pos) {
  let start,
    end,
    node = {
      type: "TSTypeParameterInstantiation",
      params: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.params = deserializeVecTSType(pos + 16);
  return node;
}

function deserializeTSTypeParameter(pos) {
  let start,
    end,
    node = {
      type: "TSTypeParameter",
      name: null,
      constraint: null,
      default: null,
      in: uint8[pos + 12] === 1,
      out: uint8[pos + 13] === 1,
      const: uint8[pos + 14] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.name = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeOptionTSType(pos + 48);
  node.default = deserializeOptionTSType(pos + 64);
  return node;
}

function deserializeTSTypeParameterDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSTypeParameterDeclaration",
      params: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.params = deserializeVecTSTypeParameter(pos + 16);
  return node;
}

function deserializeTSTypeAliasDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSTypeAliasDeclaration",
      id: null,
      typeParameters: null,
      typeAnnotation: null,
      declare: uint8[pos + 72] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.typeAnnotation = deserializeTSType(pos + 56);
  return node;
}

function deserializeTSInterfaceDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSInterfaceDeclaration",
      id: null,
      typeParameters: null,
      extends: null,
      body: null,
      declare: uint8[pos + 88] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.extends = deserializeVecTSInterfaceHeritage(pos + 56);
  node.body = deserializeTSInterfaceBody(int32[(pos + 80) >> 2]);
  return node;
}

function deserializeTSInterfaceBody(pos) {
  let start,
    end,
    node = {
      type: "TSInterfaceBody",
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.body = deserializeVecTSSignature(pos + 16);
  return node;
}

function deserializeTSPropertySignature(pos) {
  let start,
    end,
    node = {
      type: "TSPropertySignature",
      computed: uint8[pos + 12] === 1,
      optional: uint8[pos + 13] === 1,
      readonly: uint8[pos + 14] === 1,
      key: null,
      typeAnnotation: null,
      accessibility: null,
      static: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.key = deserializePropertyKey(pos + 16);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  return node;
}

function deserializeTSSignature(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSIndexSignature(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSPropertySignature(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSCallSignatureDeclaration(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSConstructSignatureDeclaration(int32[(pos + 8) >> 2]);
    case 4:
      return deserializeTSMethodSignature(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSSignature`);
  }
}

function deserializeTSIndexSignature(pos) {
  let start,
    end,
    node = {
      type: "TSIndexSignature",
      parameters: null,
      typeAnnotation: null,
      readonly: uint8[pos + 12] === 1,
      static: uint8[pos + 13] === 1,
      accessibility: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.parameters = [deserializeTSIndexSignatureName(pos + 16)];
  node.typeAnnotation = deserializeTSTypeAnnotation(int32[(pos + 56) >> 2]);
  return node;
}

function deserializeTSCallSignatureDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSCallSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    params = deserializeFormalParameters(int32[(pos + 32) >> 2]),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 40);
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
    node = {
      type: "TSMethodSignature",
      key: null,
      computed: uint8[pos + 64] === 1,
      optional: uint8[pos + 65] === 1,
      kind: deserializeTSMethodSignatureKind(pos + 66),
      typeParameters: null,
      params: null,
      returnType: null,
      accessibility: null,
      readonly: false,
      static: false,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    params = deserializeFormalParameters(int32[(pos + 48) >> 2]),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 40);
  thisParam !== null && params.unshift(thisParam);
  node.key = deserializePropertyKey(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 32);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  return node;
}

function deserializeTSConstructSignatureDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSConstructSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos + 24) >> 2]);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  return node;
}

function deserializeTSIndexSignatureName(pos) {
  let start,
    end,
    node = {
      type: "Identifier",
      decorators: [],
      name: deserializeStr(pos + 16),
      optional: false,
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSTypeAnnotation(int32[(pos + 32) >> 2]);
  return node;
}

function deserializeTSInterfaceHeritage(pos) {
  let start,
    end,
    node = {
      type: "TSInterfaceHeritage",
      expression: null,
      typeArguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
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
      });
    for (; object.type === "TSQualifiedName";) {
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
      };
      object = left;
    }
  }
  node.expression = expression;
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  return node;
}

function deserializeTSTypePredicate(pos) {
  let start,
    end,
    node = {
      type: "TSTypePredicate",
      parameterName: null,
      asserts: uint8[pos + 12] === 1,
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.parameterName = deserializeTSTypePredicateName(pos + 16);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  return node;
}

function deserializeTSTypePredicateName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSThisType(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypePredicateName`);
  }
}

function deserializeTSExternalModuleDeclaration(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    declare = uint8[pos + 72] === 1,
    body = deserializeOptionBoxTSModuleBlock(pos + 64),
    node = {
      type: "TSModuleDeclaration",
      id: null,
      ...(body !== null && { body }),
      kind: "module",
      declare,
      global: false,
      start,
      end,
      range: [start, end],
    };
  node.id = deserializeStringLiteral(pos + 16);
  return node;
}

function deserializeTSNamespaceDeclaration(pos) {
  let kind = deserializeTSNamespaceDeclarationKind(pos + 64),
    start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    declare = uint8[pos + 65] === 1,
    body = deserializeTSNamespaceDeclarationBody(pos + 48),
    node = {
      type: "TSModuleDeclaration",
      id: null,
      body,
      kind,
      declare,
      global: false,
      start,
      end,
      range: [start, end],
    },
    id = deserializeBindingIdentifier(pos + 16);
  if (body.type === "TSModuleBlock") node.id = id;
  else {
    let innerId = body.id;
    if (innerId.type === "Identifier") {
      let start, end;
      node.id = {
        type: "TSQualifiedName",
        left: id,
        right: innerId,
        start: (start = id.start),
        end: (end = innerId.end),
        range: [start, end],
      };
    } else {
      // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with
      // `id` of this namespace on left, and previous `left` on right.
      node.id = innerId;
      let { start } = id;
      for (;;) {
        innerId.start = innerId.range[0] = start;
        if (innerId.left.type === "Identifier") break;
        innerId = innerId.left;
      }
      let end,
        right = innerId.left;
      innerId.left = {
        type: "TSQualifiedName",
        left: id,
        right,
        start,
        end: (end = right.end),
        range: [start, end],
      };
    }
    body = body.body;
    node.body = body;
  }
  return node;
}

function deserializeTSNamespaceDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "module";
    case 1:
      return "namespace";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSNamespaceDeclarationKind`);
  }
}

function deserializeTSNamespaceDeclarationBody(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSNamespaceDeclaration(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSModuleBlock(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSNamespaceDeclarationBody`);
  }
}

function deserializeTSGlobalDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSModuleDeclaration",
      id: null,
      body: null,
      kind: "global",
      declare: uint8[pos + 88] === 1,
      global: true,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    keywordStart,
    keywordEnd;
  node.id = {
    type: "Identifier",
    name: "global",
    start: (keywordStart = int32[(pos + 16) >> 2]),
    end: (keywordEnd = int32[(pos + 20) >> 2]),
    range: [keywordStart, keywordEnd],
  };
  node.body = deserializeTSModuleBlock(pos + 24);
  return node;
}

function deserializeTSModuleBlock(pos) {
  let start,
    end,
    node = {
      type: "TSModuleBlock",
      body: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    body = deserializeVecDirective(pos + 16);
  body = appendArray(body, deserializeVecStatement(pos + 40));
  node.body = body;
  return node;
}

function deserializeTSTypeLiteral(pos) {
  let start,
    end,
    node = {
      type: "TSTypeLiteral",
      members: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.members = deserializeVecTSSignature(pos + 16);
  return node;
}

function deserializeTSInferType(pos) {
  let start,
    end,
    node = {
      type: "TSInferType",
      typeParameter: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeParameter = deserializeTSTypeParameter(int32[(pos + 16) >> 2]);
  return node;
}

function deserializeTSTypeQuery(pos) {
  let start,
    end,
    node = {
      type: "TSTypeQuery",
      exprName: null,
      typeArguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.exprName = deserializeTSTypeQueryExprName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  return node;
}

function deserializeTSTypeQueryExprName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSQualifiedName(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeThisExpression(int32[(pos + 8) >> 2]);
    case 3:
      return deserializeTSImportType(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeQueryExprName`);
  }
}

function deserializeTSImportType(pos) {
  let start,
    end,
    node = {
      type: "TSImportType",
      source: null,
      options: null,
      qualifier: null,
      typeArguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.source = deserializeStringLiteral(pos + 16);
  node.options = deserializeOptionBoxObjectExpression(pos + 64);
  node.qualifier = deserializeOptionTSImportTypeQualifier(pos + 72);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 88);
  return node;
}

function deserializeTSImportTypeQualifier(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeTSImportTypeQualifiedName(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSImportTypeQualifier`);
  }
}

function deserializeTSImportTypeQualifiedName(pos) {
  let start,
    end,
    node = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.left = deserializeTSImportTypeQualifier(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  return node;
}

function deserializeTSFunctionType(pos) {
  let start,
    end,
    node = {
      type: "TSFunctionType",
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    params = deserializeFormalParameters(int32[(pos + 32) >> 2]),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = params;
  node.returnType = deserializeTSTypeAnnotation(int32[(pos + 40) >> 2]);
  return node;
}

function deserializeTSConstructorType(pos) {
  let start,
    end,
    node = {
      type: "TSConstructorType",
      abstract: uint8[pos + 40] === 1,
      typeParameters: null,
      params: null,
      returnType: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos + 24) >> 2]);
  node.returnType = deserializeTSTypeAnnotation(int32[(pos + 32) >> 2]);
  return node;
}

function deserializeTSMappedType(pos) {
  let start,
    end,
    node = {
      type: "TSMappedType",
      key: null,
      constraint: null,
      nameType: null,
      typeAnnotation: null,
      optional: null,
      readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 97),
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    },
    optional = deserializeOptionTSMappedTypeModifierOperator(pos + 96);
  optional === null && (optional = false);
  node.key = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeTSType(pos + 48);
  node.nameType = deserializeOptionTSType(pos + 64);
  node.typeAnnotation = deserializeOptionTSType(pos + 80);
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
  let start,
    end,
    node = {
      type: "TSTemplateLiteralType",
      quasis: null,
      types: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.types = deserializeVecTSType(pos + 40);
  return node;
}

function deserializeTSAsExpression(pos) {
  let start,
    end,
    node = {
      type: "TSAsExpression",
      expression: null,
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  return node;
}

function deserializeTSSatisfiesExpression(pos) {
  let start,
    end,
    node = {
      type: "TSSatisfiesExpression",
      expression: null,
      typeAnnotation: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  return node;
}

function deserializeTSTypeAssertion(pos) {
  let start,
    end,
    node = {
      type: "TSTypeAssertion",
      typeAnnotation: null,
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
  node.expression = deserializeExpression(pos + 32);
  return node;
}

function deserializeTSImportEqualsDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSImportEqualsDeclaration",
      id: null,
      moduleReference: null,
      importKind: deserializeImportOrExportKind(pos + 12),
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeBindingIdentifier(pos + 16);
  node.moduleReference = deserializeTSModuleReference(pos + 48);
  return node;
}

function deserializeTSModuleReference(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSExternalModuleReference(int32[(pos + 8) >> 2]);
    case 1:
      return deserializeIdentifierReference(int32[(pos + 8) >> 2]);
    case 2:
      return deserializeTSQualifiedName(int32[(pos + 8) >> 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleReference`);
  }
}

function deserializeTSExternalModuleReference(pos) {
  let start,
    end,
    node = {
      type: "TSExternalModuleReference",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeStringLiteral(pos + 16);
  return node;
}

function deserializeTSNonNullExpression(pos) {
  let start,
    end,
    node = {
      type: "TSNonNullExpression",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  return node;
}

function deserializeDecorator(pos) {
  let start,
    end,
    node = {
      type: "Decorator",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  return node;
}

function deserializeTSExportAssignment(pos) {
  let start,
    end,
    node = {
      type: "TSExportAssignment",
      expression: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  return node;
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let start,
    end,
    node = {
      type: "TSNamespaceExportDeclaration",
      id: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.id = deserializeIdentifierName(pos + 16);
  return node;
}

function deserializeTSInstantiationExpression(pos) {
  let start,
    end,
    node = {
      type: "TSInstantiationExpression",
      expression: null,
      typeArguments: null,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.expression = deserializeExpression(pos + 16);
  node.typeArguments = deserializeTSTypeParameterInstantiation(int32[(pos + 32) >> 2]);
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
    node = {
      type: "TSJSDocNullableType",
      typeAnnotation: null,
      postfix: uint8[pos + 12] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
  return node;
}

function deserializeJSDocNonNullableType(pos) {
  let start,
    end,
    node = {
      type: "TSJSDocNonNullableType",
      typeAnnotation: null,
      postfix: uint8[pos + 12] === 1,
      start: (start = int32[pos >> 2]),
      end: (end = int32[(pos + 4) >> 2]),
      range: [start, end],
    };
  node.typeAnnotation = deserializeTSType(pos + 16);
  return node;
}

function deserializeJSDocUnknownType(pos) {
  let start, end;
  return {
    type: "TSJSDocUnknownType",
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
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
    case 3:
      return "Line";
    case 4:
      return "Line";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for CommentKind`);
  }
}

function deserializeComment(pos) {
  let type = deserializeCommentKind(pos + 17),
    start = int32[pos >> 2],
    end = int32[(pos + 4) >> 2],
    kind = uint8[pos + 17];
  return {
    type,
    value: sourceText.slice(start + (kind < 3 ? 2 : kind), end - (type === "Line" ? 0 : 2)),
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
    start: int32[pos >> 2],
    end: int32[(pos + 4) >> 2],
  };
}

function deserializeNameSpan(pos) {
  let start, end;
  return {
    value: deserializeStr(pos + 8),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeImportEntry(pos) {
  return {
    importName: deserializeImportImportName(pos + 32),
    localName: deserializeNameSpan(pos + 64),
    isType: uint8[pos + 88] === 1,
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
    isType: uint8[pos + 136] === 1,
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
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
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
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
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeEcmaScriptModule(pos) {
  return {
    hasModuleSyntax: uint8[pos + 96] === 1,
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
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeStaticExport(pos) {
  let start, end;
  return {
    entries: deserializeVecExportEntry(pos + 8),
    start: (start = int32[pos >> 2]),
    end: (end = int32[(pos + 4) >> 2]),
    range: [start, end],
  };
}

function deserializeStr(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return "";
  pos = int32[pos32];
  let end = pos + len,
    isInSourceRegion = pos >= sourceStartPos;
  if (isInSourceRegion && end <= firstNonAsciiPos)
    return sourceTextLatin.substr(pos - sourceStartPos, len);
  // Use `utf8Slice` for strings longer than 64 bytes
  if (len > 64) return utf8Slice.call(uint8, pos, end);
  // If string is in source region, use slice of `sourceTextLatin` if all ASCII
  if (isInSourceRegion) {
    // Check if all bytes are ASCII, use `utf8Slice` if not
    for (let i = pos; i < end; i++) if (uint8[i] >= 128) return utf8Slice.call(uint8, pos, end);
    // String is all ASCII, so slice from `sourceTextLatin`
    return sourceTextLatin.substr(pos - sourceStartPos, len);
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
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeComment(pos);
    pos += 24;
  }
  return arr;
}

function deserializeOptionHashbang(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeHashbang(pos);
}

function deserializeVecDirective(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeDirective(pos);
    pos += 80;
  }
  return arr;
}

function deserializeVecStatement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeStatement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecArrayExpressionElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeArrayExpressionElement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecObjectPropertyKind(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeObjectPropertyKind(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecTemplateElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTemplateElement(pos);
    pos += 48;
  }
  return arr;
}

function deserializeVecExpression(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeExpression(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxTSTypeParameterInstantiation(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSTypeParameterInstantiation(int32[pos >> 2]);
}

function deserializeOptionStr(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0 ? null : deserializeStr(pos);
}

function deserializeVecArgument(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeArgument(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionAssignmentTargetMaybeDefault(pos) {
  return uint8[pos] === 255 ? null : deserializeAssignmentTargetMaybeDefault(pos);
}

function deserializeVecOptionAssignmentTargetMaybeDefault(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeOptionAssignmentTargetMaybeDefault(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxAssignmentTargetRest(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeAssignmentTargetRest(int32[pos >> 2]);
}

function deserializeVecAssignmentTargetProperty(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeAssignmentTargetProperty(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionExpression(pos) {
  return uint8[pos] === 255 ? null : deserializeExpression(pos);
}

function deserializeVecVariableDeclarator(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeVariableDeclarator(pos);
    pos += 56;
  }
  return arr;
}

function deserializeOptionBoxTSTypeAnnotation(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSTypeAnnotation(int32[pos >> 2]);
}

function deserializeOptionStatement(pos) {
  return uint8[pos] === 255 ? null : deserializeStatement(pos);
}

function deserializeOptionForStatementInit(pos) {
  return uint8[pos] === 255 ? null : deserializeForStatementInit(pos);
}

function deserializeOptionLabelIdentifier(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeLabelIdentifier(pos);
}

function deserializeVecSwitchCase(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeSwitchCase(pos);
    pos += 56;
  }
  return arr;
}

function deserializeOptionBoxCatchClause(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeCatchClause(int32[pos >> 2]);
}

function deserializeOptionBoxBlockStatement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBlockStatement(int32[pos >> 2]);
}

function deserializeOptionCatchParameter(pos) {
  return uint8[pos + 16] === 255 ? null : deserializeCatchParameter(pos);
}

function deserializeVecBindingProperty(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeBindingProperty(pos);
    pos += 48;
  }
  return arr;
}

function deserializeOptionBoxBindingRestElement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBindingRestElement(int32[pos >> 2]);
}

function deserializeOptionBindingPattern(pos) {
  return uint8[pos] === 255 ? null : deserializeBindingPattern(pos);
}

function deserializeVecOptionBindingPattern(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeOptionBindingPattern(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBindingIdentifier(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeBindingIdentifier(pos);
}

function deserializeOptionBoxTSTypeParameterDeclaration(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSTypeParameterDeclaration(int32[pos >> 2]);
}

function deserializeOptionBoxTSThisParameter(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSThisParameter(int32[pos >> 2]);
}

function deserializeOptionBoxFunctionBody(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeFunctionBody(int32[pos >> 2]);
}

function deserializeVecFormalParameter(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeFormalParameter(pos);
    pos += 72;
  }
  return arr;
}

function deserializeVecDecorator(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeDecorator(pos);
    pos += 32;
  }
  return arr;
}

function deserializeOptionBoxExpression(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeExpression(int32[pos >> 2]);
}

function deserializeVecClassElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeClassElement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionImportPhase(pos) {
  return uint8[pos] === 2 ? null : deserializeImportPhase(pos);
}

function deserializeVecImportDeclarationSpecifier(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeImportDeclarationSpecifier(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionVecImportDeclarationSpecifier(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeVecImportDeclarationSpecifier(pos);
}

function deserializeOptionBoxWithClause(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeWithClause(int32[pos >> 2]);
}

function deserializeVecImportAttribute(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeImportAttribute(pos);
    pos += 120;
  }
  return arr;
}

function deserializeVecExportSpecifier(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeExportSpecifier(pos);
    pos += 128;
  }
  return arr;
}

function deserializeOptionModuleExportName(pos) {
  return uint8[pos] === 255 ? null : deserializeModuleExportName(pos);
}

function deserializeVecJSXChild(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeJSXChild(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxJSXClosingElement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeJSXClosingElement(int32[pos >> 2]);
}

function deserializeVecJSXAttributeItem(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeJSXAttributeItem(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionJSXAttributeValue(pos) {
  return uint8[pos] === 255 ? null : deserializeJSXAttributeValue(pos);
}

function deserializeVecTSEnumMember(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSEnumMember(pos);
    pos += 48;
  }
  return arr;
}

function deserializeVecTSType(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSType(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecTSTupleElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSTupleElement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionTSType(pos) {
  return uint8[pos] === 255 ? null : deserializeTSType(pos);
}

function deserializeVecTSTypeParameter(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSTypeParameter(pos);
    pos += 80;
  }
  return arr;
}

function deserializeVecTSInterfaceHeritage(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSInterfaceHeritage(pos);
    pos += 40;
  }
  return arr;
}

function deserializeVecTSSignature(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSSignature(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxTSModuleBlock(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSModuleBlock(int32[pos >> 2]);
}

function deserializeOptionBoxObjectExpression(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeObjectExpression(int32[pos >> 2]);
}

function deserializeOptionTSImportTypeQualifier(pos) {
  return uint8[pos] === 2 ? null : deserializeTSImportTypeQualifier(pos);
}

function deserializeOptionTSMappedTypeModifierOperator(pos) {
  return uint8[pos] === 255 ? null : deserializeTSMappedTypeModifierOperator(pos);
}

function deserializeOptionNameSpan(pos) {
  return int32[(pos >> 2) + 2] === 0 && int32[(pos >> 2) + 3] === 0
    ? null
    : deserializeNameSpan(pos);
}

function deserializeVecError(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeError(pos);
    pos += 80;
  }
  return arr;
}

function deserializeVecErrorLabel(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeErrorLabel(pos);
    pos += 24;
  }
  return arr;
}

function deserializeVecStaticImport(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeStaticImport(pos);
    pos += 56;
  }
  return arr;
}

function deserializeVecStaticExport(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeStaticExport(pos);
    pos += 32;
  }
  return arr;
}

function deserializeVecDynamicImport(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeDynamicImport(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecSpan(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeSpan(pos);
    pos += 8;
  }
  return arr;
}

function deserializeVecImportEntry(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeImportEntry(pos);
    pos += 96;
  }
  return arr;
}

function deserializeVecExportEntry(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2],
    arr = Array(len);
  pos = int32[pos32];
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeExportEntry(pos);
    pos += 144;
  }
  return arr;
}
