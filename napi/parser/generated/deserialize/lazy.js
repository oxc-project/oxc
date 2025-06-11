// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer_lazy.rs`.

'use strict';

// Unique token which is not exposed publicly.
// Used to prevent user calling class constructors.
const TOKEN = {};

module.exports = { construct, TOKEN };

function construct(ast) {
  // (2 * 1024 * 1024 * 1024 - 16) >> 2
  const metadataPos32 = 536870908;

  return new RawTransferData(ast.buffer.uint32[metadataPos32], ast, TOKEN);
}

const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true }),
  decodeStr = textDecoder.decode.bind(textDecoder),
  { fromCodePoint } = String;

class Program {
  type = 'Program';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get sourceType() {
    const internal = this.#internal;
    return new SourceType(internal.$pos + 124, internal.$ast);
  }

  get hashbang() {
    const internal = this.#internal;
    return constructOptionHashbang(internal.$pos + 48, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecStatement(internal.$pos + 96, internal.$ast);
  }

  toJSON() {
    return {
      type: 'Program',
      start: this.start,
      end: this.end,
      sourceType: this.sourceType,
      hashbang: this.hashbang,
      body: this.body,
    };
  }
}

function constructExpression(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Expression`);
  }
}

class IdentifierName {
  type = 'IdentifierName';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'IdentifierName',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

class IdentifierReference {
  type = 'IdentifierReference';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'IdentifierReference',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

class BindingIdentifier {
  type = 'BindingIdentifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BindingIdentifier',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

class LabelIdentifier {
  type = 'LabelIdentifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'LabelIdentifier',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

class ThisExpression {
  type = 'ThisExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ThisExpression',
      start: this.start,
      end: this.end,
    };
  }
}

class ArrayExpression {
  type = 'ArrayExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, elements: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get elements() {
    const internal = this.#internal,
      node = internal.elements;
    if (node !== void 0) return node;
    return internal.elements = constructVecArrayExpressionElement(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ArrayExpression',
      start: this.start,
      end: this.end,
      elements: this.elements,
    };
  }
}

function constructArrayExpressionElement(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    case 64:
      return constructBoxSpreadElement(pos + 8, ast);
    case 65:
      return new Elision(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ArrayExpressionElement`);
  }
}

class Elision {
  type = 'Elision';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'Elision',
      start: this.start,
      end: this.end,
    };
  }
}

class ObjectExpression {
  type = 'ObjectExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, properties: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get properties() {
    const internal = this.#internal,
      node = internal.properties;
    if (node !== void 0) return node;
    return internal.properties = constructVecObjectPropertyKind(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ObjectExpression',
      start: this.start,
      end: this.end,
      properties: this.properties,
    };
  }
}

function constructObjectPropertyKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxObjectProperty(pos + 8, ast);
    case 1:
      return constructBoxSpreadElement(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ObjectPropertyKind`);
  }
}

class ObjectProperty {
  type = 'ObjectProperty';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructPropertyKind(internal.$pos + 40, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 8, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  get method() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 41, internal.$ast);
  }

  get shorthand() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 42, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 43, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ObjectProperty',
      start: this.start,
      end: this.end,
      kind: this.kind,
      key: this.key,
      value: this.value,
      method: this.method,
      shorthand: this.shorthand,
      computed: this.computed,
    };
  }
}

function constructPropertyKey(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    case 64:
      return constructBoxIdentifierName(pos + 8, ast);
    case 65:
      return constructBoxPrivateIdentifier(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyKey`);
  }
}

function constructPropertyKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'init';
    case 1:
      return 'get';
    case 2:
      return 'set';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyKind`);
  }
}

class TemplateLiteral {
  type = 'TemplateLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, quasis: void 0, expressions: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get quasis() {
    const internal = this.#internal,
      node = internal.quasis;
    if (node !== void 0) return node;
    return internal.quasis = constructVecTemplateElement(internal.$pos + 8, internal.$ast);
  }

  get expressions() {
    const internal = this.#internal,
      node = internal.expressions;
    if (node !== void 0) return node;
    return internal.expressions = constructVecExpression(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TemplateLiteral',
      start: this.start,
      end: this.end,
      quasis: this.quasis,
      expressions: this.expressions,
    };
  }
}

class TaggedTemplateExpression {
  type = 'TaggedTemplateExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get tag() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  get quasi() {
    const internal = this.#internal;
    return new TemplateLiteral(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TaggedTemplateExpression',
      start: this.start,
      end: this.end,
      tag: this.tag,
      typeArguments: this.typeArguments,
      quasi: this.quasi,
    };
  }
}

class TemplateElement {
  type = 'TemplateElement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return new TemplateElementValue(internal.$pos + 8, internal.$ast);
  }

  get tail() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TemplateElement',
      start: this.start,
      end: this.end,
      value: this.value,
      tail: this.tail,
    };
  }
}

class TemplateElementValue {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, raw: void 0, cooked: void 0 };
  }

  get raw() {
    const internal = this.#internal,
      node = internal.raw;
    if (node !== void 0) return node;
    return internal.raw = constructStr(internal.$pos, internal.$ast);
  }

  get cooked() {
    const internal = this.#internal,
      node = internal.cooked;
    if (node !== void 0) return node;
    return internal.cooked = constructOptionStr(internal.$pos + 16, internal.$ast);
  }

  toJSON() {
    return {
      raw: this.raw,
      cooked: this.cooked,
    };
  }
}

function constructMemberExpression(pos, ast) {
  switch (ast.buffer[pos]) {
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for MemberExpression`);
  }
}

class ComputedMemberExpression {
  type = 'ComputedMemberExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get property() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ComputedMemberExpression',
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
      optional: this.optional,
    };
  }
}

class StaticMemberExpression {
  type = 'StaticMemberExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get property() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 24, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 48, internal.$ast);
  }

  toJSON() {
    return {
      type: 'StaticMemberExpression',
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
      optional: this.optional,
    };
  }
}

class PrivateFieldExpression {
  type = 'PrivateFieldExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get property() {
    const internal = this.#internal;
    return new PrivateIdentifier(internal.$pos + 24, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 48, internal.$ast);
  }

  toJSON() {
    return {
      type: 'PrivateFieldExpression',
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
      optional: this.optional,
    };
  }
}

class CallExpression {
  type = 'CallExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, arguments: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get callee() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  get arguments() {
    const internal = this.#internal,
      node = internal.arguments;
    if (node !== void 0) return node;
    return internal.arguments = constructVecArgument(internal.$pos + 32, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 56, internal.$ast);
  }

  toJSON() {
    return {
      type: 'CallExpression',
      start: this.start,
      end: this.end,
      callee: this.callee,
      typeArguments: this.typeArguments,
      arguments: this.arguments,
      optional: this.optional,
    };
  }
}

class NewExpression {
  type = 'NewExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, arguments: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get callee() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  get arguments() {
    const internal = this.#internal,
      node = internal.arguments;
    if (node !== void 0) return node;
    return internal.arguments = constructVecArgument(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'NewExpression',
      start: this.start,
      end: this.end,
      callee: this.callee,
      typeArguments: this.typeArguments,
      arguments: this.arguments,
    };
  }
}

class MetaProperty {
  type = 'MetaProperty';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get meta() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 8, internal.$ast);
  }

  get property() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'MetaProperty',
      start: this.start,
      end: this.end,
      meta: this.meta,
      property: this.property,
    };
  }
}

class SpreadElement {
  type = 'SpreadElement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'SpreadElement',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

function constructArgument(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    case 64:
      return constructBoxSpreadElement(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Argument`);
  }
}

class UpdateExpression {
  type = 'UpdateExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructUpdateOperator(internal.$pos + 24, internal.$ast);
  }

  get prefix() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 25, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructSimpleAssignmentTarget(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'UpdateExpression',
      start: this.start,
      end: this.end,
      operator: this.operator,
      prefix: this.prefix,
      argument: this.argument,
    };
  }
}

class UnaryExpression {
  type = 'UnaryExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructUnaryOperator(internal.$pos + 24, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'UnaryExpression',
      start: this.start,
      end: this.end,
      operator: this.operator,
      argument: this.argument,
    };
  }
}

class BinaryExpression {
  type = 'BinaryExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructBinaryOperator(internal.$pos + 40, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BinaryExpression',
      start: this.start,
      end: this.end,
      left: this.left,
      operator: this.operator,
      right: this.right,
    };
  }
}

class PrivateInExpression {
  type = 'PrivateInExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return new PrivateIdentifier(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'PrivateInExpression',
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }
}

class LogicalExpression {
  type = 'LogicalExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructLogicalOperator(internal.$pos + 40, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'LogicalExpression',
      start: this.start,
      end: this.end,
      left: this.left,
      operator: this.operator,
      right: this.right,
    };
  }
}

class ConditionalExpression {
  type = 'ConditionalExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get consequent() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  get alternate() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ConditionalExpression',
      start: this.start,
      end: this.end,
      test: this.test,
      consequent: this.consequent,
      alternate: this.alternate,
    };
  }
}

class AssignmentExpression {
  type = 'AssignmentExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructAssignmentOperator(internal.$pos + 40, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructAssignmentTarget(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AssignmentExpression',
      start: this.start,
      end: this.end,
      operator: this.operator,
      left: this.left,
      right: this.right,
    };
  }
}

function constructAssignmentTarget(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 2:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 3:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 4:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 8:
      return constructBoxArrayAssignmentTarget(pos + 8, ast);
    case 9:
      return constructBoxObjectAssignmentTarget(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTarget`);
  }
}

function constructSimpleAssignmentTarget(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 2:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 3:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 4:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for SimpleAssignmentTarget`);
  }
}

function constructAssignmentTargetPattern(pos, ast) {
  switch (ast.buffer[pos]) {
    case 8:
      return constructBoxArrayAssignmentTarget(pos + 8, ast);
    case 9:
      return constructBoxObjectAssignmentTarget(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetPattern`);
  }
}

class ArrayAssignmentTarget {
  type = 'ArrayAssignmentTarget';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, elements: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get elements() {
    const internal = this.#internal,
      node = internal.elements;
    if (node !== void 0) return node;
    return internal.elements = constructVecOptionAssignmentTargetMaybeDefault(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ArrayAssignmentTarget',
      start: this.start,
      end: this.end,
      elements: this.elements,
    };
  }
}

class ObjectAssignmentTarget {
  type = 'ObjectAssignmentTarget';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, properties: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get properties() {
    const internal = this.#internal,
      node = internal.properties;
    if (node !== void 0) return node;
    return internal.properties = constructVecAssignmentTargetProperty(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ObjectAssignmentTarget',
      start: this.start,
      end: this.end,
      properties: this.properties,
    };
  }
}

class AssignmentTargetRest {
  type = 'AssignmentTargetRest';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructAssignmentTarget(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AssignmentTargetRest',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

function constructAssignmentTargetMaybeDefault(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 2:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 3:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 4:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 8:
      return constructBoxArrayAssignmentTarget(pos + 8, ast);
    case 9:
      return constructBoxObjectAssignmentTarget(pos + 8, ast);
    case 16:
      return constructBoxAssignmentTargetWithDefault(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetMaybeDefault`);
  }
}

class AssignmentTargetWithDefault {
  type = 'AssignmentTargetWithDefault';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructAssignmentTarget(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AssignmentTargetWithDefault',
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }
}

function constructAssignmentTargetProperty(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxAssignmentTargetPropertyIdentifier(pos + 8, ast);
    case 1:
      return constructBoxAssignmentTargetPropertyProperty(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetProperty`);
  }
}

class AssignmentTargetPropertyIdentifier {
  type = 'AssignmentTargetPropertyIdentifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return new IdentifierReference(internal.$pos + 8, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AssignmentTargetPropertyIdentifier',
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
    };
  }
}

class AssignmentTargetPropertyProperty {
  type = 'AssignmentTargetPropertyProperty';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 8, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructAssignmentTargetMaybeDefault(internal.$pos + 24, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AssignmentTargetPropertyProperty',
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
      computed: this.computed,
    };
  }
}

class SequenceExpression {
  type = 'SequenceExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, expressions: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expressions() {
    const internal = this.#internal,
      node = internal.expressions;
    if (node !== void 0) return node;
    return internal.expressions = constructVecExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'SequenceExpression',
      start: this.start,
      end: this.end,
      expressions: this.expressions,
    };
  }
}

class Super {
  type = 'Super';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'Super',
      start: this.start,
      end: this.end,
    };
  }
}

class AwaitExpression {
  type = 'AwaitExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AwaitExpression',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

class ChainExpression {
  type = 'ChainExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructChainElement(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ChainExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

function constructChainElement(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxCallExpression(pos + 8, ast);
    case 1:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ChainElement`);
  }
}

class ParenthesizedExpression {
  type = 'ParenthesizedExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ParenthesizedExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

function constructStatement(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBlockStatement(pos + 8, ast);
    case 1:
      return constructBoxBreakStatement(pos + 8, ast);
    case 2:
      return constructBoxContinueStatement(pos + 8, ast);
    case 3:
      return constructBoxDebuggerStatement(pos + 8, ast);
    case 4:
      return constructBoxDoWhileStatement(pos + 8, ast);
    case 5:
      return constructBoxEmptyStatement(pos + 8, ast);
    case 6:
      return constructBoxExpressionStatement(pos + 8, ast);
    case 7:
      return constructBoxForInStatement(pos + 8, ast);
    case 8:
      return constructBoxForOfStatement(pos + 8, ast);
    case 9:
      return constructBoxForStatement(pos + 8, ast);
    case 10:
      return constructBoxIfStatement(pos + 8, ast);
    case 11:
      return constructBoxLabeledStatement(pos + 8, ast);
    case 12:
      return constructBoxReturnStatement(pos + 8, ast);
    case 13:
      return constructBoxSwitchStatement(pos + 8, ast);
    case 14:
      return constructBoxThrowStatement(pos + 8, ast);
    case 15:
      return constructBoxTryStatement(pos + 8, ast);
    case 16:
      return constructBoxWhileStatement(pos + 8, ast);
    case 17:
      return constructBoxWithStatement(pos + 8, ast);
    case 32:
      return constructBoxVariableDeclaration(pos + 8, ast);
    case 33:
      return constructBoxFunction(pos + 8, ast);
    case 34:
      return constructBoxClass(pos + 8, ast);
    case 35:
      return constructBoxTSTypeAliasDeclaration(pos + 8, ast);
    case 36:
      return constructBoxTSInterfaceDeclaration(pos + 8, ast);
    case 37:
      return constructBoxTSEnumDeclaration(pos + 8, ast);
    case 38:
      return constructBoxTSModuleDeclaration(pos + 8, ast);
    case 39:
      return constructBoxTSImportEqualsDeclaration(pos + 8, ast);
    case 64:
      return constructBoxImportDeclaration(pos + 8, ast);
    case 65:
      return constructBoxExportAllDeclaration(pos + 8, ast);
    case 66:
      return constructBoxExportDefaultDeclaration(pos + 8, ast);
    case 67:
      return constructBoxExportNamedDeclaration(pos + 8, ast);
    case 68:
      return constructBoxTSExportAssignment(pos + 8, ast);
    case 69:
      return constructBoxTSNamespaceExportDeclaration(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Statement`);
  }
}

class Directive {
  type = 'Directive';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, directive: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return new StringLiteral(internal.$pos + 8, internal.$ast);
  }

  get directive() {
    const internal = this.#internal,
      node = internal.directive;
    if (node !== void 0) return node;
    return internal.directive = constructStr(internal.$pos + 56, internal.$ast);
  }

  toJSON() {
    return {
      type: 'Directive',
      start: this.start,
      end: this.end,
      expression: this.expression,
      directive: this.directive,
    };
  }
}

class Hashbang {
  type = 'Hashbang';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, value: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal,
      node = internal.value;
    if (node !== void 0) return node;
    return internal.value = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'Hashbang',
      start: this.start,
      end: this.end,
      value: this.value,
    };
  }
}

class BlockStatement {
  type = 'BlockStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecStatement(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BlockStatement',
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }
}

function constructDeclaration(pos, ast) {
  switch (ast.buffer[pos]) {
    case 32:
      return constructBoxVariableDeclaration(pos + 8, ast);
    case 33:
      return constructBoxFunction(pos + 8, ast);
    case 34:
      return constructBoxClass(pos + 8, ast);
    case 35:
      return constructBoxTSTypeAliasDeclaration(pos + 8, ast);
    case 36:
      return constructBoxTSInterfaceDeclaration(pos + 8, ast);
    case 37:
      return constructBoxTSEnumDeclaration(pos + 8, ast);
    case 38:
      return constructBoxTSModuleDeclaration(pos + 8, ast);
    case 39:
      return constructBoxTSImportEqualsDeclaration(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Declaration`);
  }
}

class VariableDeclaration {
  type = 'VariableDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, declarations: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructVariableDeclarationKind(internal.$pos + 32, internal.$ast);
  }

  get declarations() {
    const internal = this.#internal,
      node = internal.declarations;
    if (node !== void 0) return node;
    return internal.declarations = constructVecVariableDeclarator(internal.$pos + 8, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 33, internal.$ast);
  }

  toJSON() {
    return {
      type: 'VariableDeclaration',
      start: this.start,
      end: this.end,
      kind: this.kind,
      declarations: this.declarations,
      declare: this.declare,
    };
  }
}

function constructVariableDeclarationKind(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for VariableDeclarationKind`);
  }
}

class VariableDeclarator {
  type = 'VariableDeclarator';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingPattern(internal.$pos + 8, internal.$ast);
  }

  get init() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 40, internal.$ast);
  }

  get definite() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 57, internal.$ast);
  }

  toJSON() {
    return {
      type: 'VariableDeclarator',
      start: this.start,
      end: this.end,
      id: this.id,
      init: this.init,
      definite: this.definite,
    };
  }
}

class EmptyStatement {
  type = 'EmptyStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'EmptyStatement',
      start: this.start,
      end: this.end,
    };
  }
}

class ExpressionStatement {
  type = 'ExpressionStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ExpressionStatement',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

class IfStatement {
  type = 'IfStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get consequent() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 24, internal.$ast);
  }

  get alternate() {
    const internal = this.#internal;
    return constructOptionStatement(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'IfStatement',
      start: this.start,
      end: this.end,
      test: this.test,
      consequent: this.consequent,
      alternate: this.alternate,
    };
  }
}

class DoWhileStatement {
  type = 'DoWhileStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 8, internal.$ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'DoWhileStatement',
      start: this.start,
      end: this.end,
      body: this.body,
      test: this.test,
    };
  }
}

class WhileStatement {
  type = 'WhileStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'WhileStatement',
      start: this.start,
      end: this.end,
      test: this.test,
      body: this.body,
    };
  }
}

class ForStatement {
  type = 'ForStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get init() {
    const internal = this.#internal;
    return constructOptionForStatementInit(internal.$pos + 8, internal.$ast);
  }

  get test() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 24, internal.$ast);
  }

  get update() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 40, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 56, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ForStatement',
      start: this.start,
      end: this.end,
      init: this.init,
      test: this.test,
      update: this.update,
      body: this.body,
    };
  }
}

function constructForStatementInit(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    case 64:
      return constructBoxVariableDeclaration(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ForStatementInit`);
  }
}

class ForInStatement {
  type = 'ForInStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructForStatementLeft(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ForInStatement',
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
      body: this.body,
    };
  }
}

function constructForStatementLeft(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 2:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 3:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 4:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 8:
      return constructBoxArrayAssignmentTarget(pos + 8, ast);
    case 9:
      return constructBoxObjectAssignmentTarget(pos + 8, ast);
    case 16:
      return constructBoxVariableDeclaration(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ForStatementLeft`);
  }
}

class ForOfStatement {
  type = 'ForOfStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get await() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 60, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructForStatementLeft(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ForOfStatement',
      start: this.start,
      end: this.end,
      await: this.await,
      left: this.left,
      right: this.right,
      body: this.body,
    };
  }
}

class ContinueStatement {
  type = 'ContinueStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get label() {
    const internal = this.#internal;
    return constructOptionLabelIdentifier(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ContinueStatement',
      start: this.start,
      end: this.end,
      label: this.label,
    };
  }
}

class BreakStatement {
  type = 'BreakStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get label() {
    const internal = this.#internal;
    return constructOptionLabelIdentifier(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BreakStatement',
      start: this.start,
      end: this.end,
      label: this.label,
    };
  }
}

class ReturnStatement {
  type = 'ReturnStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ReturnStatement',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

class WithStatement {
  type = 'WithStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'WithStatement',
      start: this.start,
      end: this.end,
      object: this.object,
      body: this.body,
    };
  }
}

class SwitchStatement {
  type = 'SwitchStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, cases: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get discriminant() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get cases() {
    const internal = this.#internal,
      node = internal.cases;
    if (node !== void 0) return node;
    return internal.cases = constructVecSwitchCase(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'SwitchStatement',
      start: this.start,
      end: this.end,
      discriminant: this.discriminant,
      cases: this.cases,
    };
  }
}

class SwitchCase {
  type = 'SwitchCase';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, consequent: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get test() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 8, internal.$ast);
  }

  get consequent() {
    const internal = this.#internal,
      node = internal.consequent;
    if (node !== void 0) return node;
    return internal.consequent = constructVecStatement(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'SwitchCase',
      start: this.start,
      end: this.end,
      test: this.test,
      consequent: this.consequent,
    };
  }
}

class LabeledStatement {
  type = 'LabeledStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get label() {
    const internal = this.#internal;
    return new LabelIdentifier(internal.$pos + 8, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'LabeledStatement',
      start: this.start,
      end: this.end,
      label: this.label,
      body: this.body,
    };
  }
}

class ThrowStatement {
  type = 'ThrowStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ThrowStatement',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

class TryStatement {
  type = 'TryStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get block() {
    const internal = this.#internal;
    return constructBoxBlockStatement(internal.$pos + 8, internal.$ast);
  }

  get handler() {
    const internal = this.#internal;
    return constructOptionBoxCatchClause(internal.$pos + 16, internal.$ast);
  }

  get finalizer() {
    const internal = this.#internal;
    return constructOptionBoxBlockStatement(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TryStatement',
      start: this.start,
      end: this.end,
      block: this.block,
      handler: this.handler,
      finalizer: this.finalizer,
    };
  }
}

class CatchClause {
  type = 'CatchClause';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get param() {
    const internal = this.#internal;
    return constructOptionCatchParameter(internal.$pos + 8, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructBoxBlockStatement(internal.$pos + 48, internal.$ast);
  }

  toJSON() {
    return {
      type: 'CatchClause',
      start: this.start,
      end: this.end,
      param: this.param,
      body: this.body,
    };
  }
}

class CatchParameter {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get pattern() {
    const internal = this.#internal;
    return new BindingPattern(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      pattern: this.pattern,
    };
  }
}

class DebuggerStatement {
  type = 'DebuggerStatement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'DebuggerStatement',
      start: this.start,
      end: this.end,
    };
  }
}

class BindingPattern {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get kind() {
    const internal = this.#internal;
    return constructBindingPatternKind(internal.$pos, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 16, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      kind: this.kind,
      typeAnnotation: this.typeAnnotation,
      optional: this.optional,
    };
  }
}

function constructBindingPatternKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBindingIdentifier(pos + 8, ast);
    case 1:
      return constructBoxObjectPattern(pos + 8, ast);
    case 2:
      return constructBoxArrayPattern(pos + 8, ast);
    case 3:
      return constructBoxAssignmentPattern(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for BindingPatternKind`);
  }
}

class AssignmentPattern {
  type = 'AssignmentPattern';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return new BindingPattern(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'AssignmentPattern',
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }
}

class ObjectPattern {
  type = 'ObjectPattern';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, properties: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get properties() {
    const internal = this.#internal,
      node = internal.properties;
    if (node !== void 0) return node;
    return internal.properties = constructVecBindingProperty(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ObjectPattern',
      start: this.start,
      end: this.end,
      properties: this.properties,
    };
  }
}

class BindingProperty {
  type = 'BindingProperty';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 8, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return new BindingPattern(internal.$pos + 24, internal.$ast);
  }

  get shorthand() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 56, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 57, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BindingProperty',
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
      shorthand: this.shorthand,
      computed: this.computed,
    };
  }
}

class ArrayPattern {
  type = 'ArrayPattern';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, elements: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get elements() {
    const internal = this.#internal,
      node = internal.elements;
    if (node !== void 0) return node;
    return internal.elements = constructVecOptionBindingPattern(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ArrayPattern',
      start: this.start,
      end: this.end,
      elements: this.elements,
    };
  }
}

class BindingRestElement {
  type = 'BindingRestElement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return new BindingPattern(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BindingRestElement',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

class Function {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get type() {
    const internal = this.#internal;
    return constructFunctionType(internal.$pos + 84, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return constructOptionBindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  get generator() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 85, internal.$ast);
  }

  get async() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 86, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 87, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 40, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 56, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 64, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructOptionBoxFunctionBody(internal.$pos + 72, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
      id: this.id,
      generator: this.generator,
      async: this.async,
      declare: this.declare,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
      body: this.body,
    };
  }
}

function constructFunctionType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'FunctionDeclaration';
    case 1:
      return 'FunctionExpression';
    case 2:
      return 'TSDeclareFunction';
    case 3:
      return 'TSEmptyBodyFunctionExpression';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for FunctionType`);
  }
}

class FormalParameters {
  type = 'FormalParameters';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, items: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructFormalParameterKind(internal.$pos + 40, internal.$ast);
  }

  get items() {
    const internal = this.#internal,
      node = internal.items;
    if (node !== void 0) return node;
    return internal.items = constructVecFormalParameter(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'FormalParameters',
      start: this.start,
      end: this.end,
      kind: this.kind,
      items: this.items,
    };
  }
}

class FormalParameter {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, decorators: void 0 };
  }

  get decorators() {
    const internal = this.#internal,
      node = internal.decorators;
    if (node !== void 0) return node;
    return internal.decorators = constructVecDecorator(internal.$pos + 8, internal.$ast);
  }

  get pattern() {
    const internal = this.#internal;
    return new BindingPattern(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      decorators: this.decorators,
      pattern: this.pattern,
    };
  }
}

function constructFormalParameterKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'FormalParameter';
    case 1:
      return 'UniqueFormalParameters';
    case 2:
      return 'ArrowFormalParameters';
    case 3:
      return 'Signature';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for FormalParameterKind`);
  }
}

class FunctionBody {
  type = 'FunctionBody';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecStatement(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'FunctionBody',
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }
}

class ArrowFunctionExpression {
  type = 'ArrowFunctionExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 44, internal.$ast);
  }

  get async() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 45, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 8, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 16, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 24, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructBoxFunctionBody(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ArrowFunctionExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
      async: this.async,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
      body: this.body,
    };
  }
}

class YieldExpression {
  type = 'YieldExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get delegate() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 24, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'YieldExpression',
      start: this.start,
      end: this.end,
      delegate: this.delegate,
      argument: this.argument,
    };
  }
}

class Class {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, decorators: void 0, implements: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get type() {
    const internal = this.#internal;
    return constructClassType(internal.$pos + 132, internal.$ast);
  }

  get decorators() {
    const internal = this.#internal,
      node = internal.decorators;
    if (node !== void 0) return node;
    return internal.decorators = constructVecDecorator(internal.$pos + 8, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return constructOptionBindingIdentifier(internal.$pos + 32, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 64, internal.$ast);
  }

  get superClass() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 72, internal.$ast);
  }

  get superTypeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 88, internal.$ast);
  }

  get implements() {
    const internal = this.#internal,
      node = internal.implements;
    if (node !== void 0) return node;
    return internal.implements = constructVecTSClassImplements(internal.$pos + 96, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructBoxClassBody(internal.$pos + 120, internal.$ast);
  }

  get abstract() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 133, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 134, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
      decorators: this.decorators,
      id: this.id,
      typeParameters: this.typeParameters,
      superClass: this.superClass,
      superTypeArguments: this.superTypeArguments,
      implements: this.implements,
      body: this.body,
      abstract: this.abstract,
      declare: this.declare,
    };
  }
}

function constructClassType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'ClassDeclaration';
    case 1:
      return 'ClassExpression';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ClassType`);
  }
}

class ClassBody {
  type = 'ClassBody';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecClassElement(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ClassBody',
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }
}

function constructClassElement(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxStaticBlock(pos + 8, ast);
    case 1:
      return constructBoxMethodDefinition(pos + 8, ast);
    case 2:
      return constructBoxPropertyDefinition(pos + 8, ast);
    case 3:
      return constructBoxAccessorProperty(pos + 8, ast);
    case 4:
      return constructBoxTSIndexSignature(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ClassElement`);
  }
}

class MethodDefinition {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, decorators: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get type() {
    const internal = this.#internal;
    return constructMethodDefinitionType(internal.$pos + 56, internal.$ast);
  }

  get decorators() {
    const internal = this.#internal,
      node = internal.decorators;
    if (node !== void 0) return node;
    return internal.decorators = constructVecDecorator(internal.$pos + 8, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 32, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructBoxFunction(internal.$pos + 48, internal.$ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructMethodDefinitionKind(internal.$pos + 57, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 58, internal.$ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 59, internal.$ast);
  }

  get override() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 60, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 61, internal.$ast);
  }

  get accessibility() {
    const internal = this.#internal;
    return constructOptionTSAccessibility(internal.$pos + 62, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
      decorators: this.decorators,
      key: this.key,
      value: this.value,
      kind: this.kind,
      computed: this.computed,
      static: this.static,
      override: this.override,
      optional: this.optional,
      accessibility: this.accessibility,
    };
  }
}

function constructMethodDefinitionType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'MethodDefinition';
    case 1:
      return 'TSAbstractMethodDefinition';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for MethodDefinitionType`);
  }
}

class PropertyDefinition {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, decorators: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get type() {
    const internal = this.#internal;
    return constructPropertyDefinitionType(internal.$pos + 72, internal.$ast);
  }

  get decorators() {
    const internal = this.#internal,
      node = internal.decorators;
    if (node !== void 0) return node;
    return internal.decorators = constructVecDecorator(internal.$pos + 8, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 32, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 48, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 56, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 73, internal.$ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 74, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 75, internal.$ast);
  }

  get override() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 76, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 77, internal.$ast);
  }

  get definite() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 78, internal.$ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 79, internal.$ast);
  }

  get accessibility() {
    const internal = this.#internal;
    return constructOptionTSAccessibility(internal.$pos + 80, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
      decorators: this.decorators,
      key: this.key,
      typeAnnotation: this.typeAnnotation,
      value: this.value,
      computed: this.computed,
      static: this.static,
      declare: this.declare,
      override: this.override,
      optional: this.optional,
      definite: this.definite,
      readonly: this.readonly,
      accessibility: this.accessibility,
    };
  }
}

function constructPropertyDefinitionType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'PropertyDefinition';
    case 1:
      return 'TSAbstractPropertyDefinition';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyDefinitionType`);
  }
}

function constructMethodDefinitionKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'constructor';
    case 1:
      return 'method';
    case 2:
      return 'get';
    case 3:
      return 'set';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for MethodDefinitionKind`);
  }
}

class PrivateIdentifier {
  type = 'PrivateIdentifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'PrivateIdentifier',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

class StaticBlock {
  type = 'StaticBlock';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecStatement(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'StaticBlock',
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }
}

function constructModuleDeclaration(pos, ast) {
  switch (ast.buffer[pos]) {
    case 64:
      return constructBoxImportDeclaration(pos + 8, ast);
    case 65:
      return constructBoxExportAllDeclaration(pos + 8, ast);
    case 66:
      return constructBoxExportDefaultDeclaration(pos + 8, ast);
    case 67:
      return constructBoxExportNamedDeclaration(pos + 8, ast);
    case 68:
      return constructBoxTSExportAssignment(pos + 8, ast);
    case 69:
      return constructBoxTSNamespaceExportDeclaration(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ModuleDeclaration`);
  }
}

function constructAccessorPropertyType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'AccessorProperty';
    case 1:
      return 'TSAbstractAccessorProperty';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AccessorPropertyType`);
  }
}

class AccessorProperty {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, decorators: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get type() {
    const internal = this.#internal;
    return constructAccessorPropertyType(internal.$pos + 72, internal.$ast);
  }

  get decorators() {
    const internal = this.#internal,
      node = internal.decorators;
    if (node !== void 0) return node;
    return internal.decorators = constructVecDecorator(internal.$pos + 8, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 32, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 48, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 56, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 73, internal.$ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 74, internal.$ast);
  }

  get override() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 75, internal.$ast);
  }

  get definite() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 76, internal.$ast);
  }

  get accessibility() {
    const internal = this.#internal;
    return constructOptionTSAccessibility(internal.$pos + 77, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
      decorators: this.decorators,
      key: this.key,
      typeAnnotation: this.typeAnnotation,
      value: this.value,
      computed: this.computed,
      static: this.static,
      override: this.override,
      definite: this.definite,
      accessibility: this.accessibility,
    };
  }
}

class ImportExpression {
  type = 'ImportExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get source() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get options() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 24, internal.$ast);
  }

  get phase() {
    const internal = this.#internal;
    return constructOptionImportPhase(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ImportExpression',
      start: this.start,
      end: this.end,
      source: this.source,
      options: this.options,
      phase: this.phase,
    };
  }
}

class ImportDeclaration {
  type = 'ImportDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get specifiers() {
    const internal = this.#internal;
    return constructOptionVecImportDeclarationSpecifier(internal.$pos + 8, internal.$ast);
  }

  get source() {
    const internal = this.#internal;
    return new StringLiteral(internal.$pos + 32, internal.$ast);
  }

  get phase() {
    const internal = this.#internal;
    return constructOptionImportPhase(internal.$pos + 88, internal.$ast);
  }

  get attributes() {
    const internal = this.#internal;
    return constructOptionBoxWithClause(internal.$pos + 80, internal.$ast);
  }

  get importKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.$pos + 89, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ImportDeclaration',
      start: this.start,
      end: this.end,
      specifiers: this.specifiers,
      source: this.source,
      phase: this.phase,
      attributes: this.attributes,
      importKind: this.importKind,
    };
  }
}

function constructImportPhase(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'source';
    case 1:
      return 'defer';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportPhase`);
  }
}

function constructImportDeclarationSpecifier(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxImportSpecifier(pos + 8, ast);
    case 1:
      return constructBoxImportDefaultSpecifier(pos + 8, ast);
    case 2:
      return constructBoxImportNamespaceSpecifier(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportDeclarationSpecifier`);
  }
}

class ImportSpecifier {
  type = 'ImportSpecifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get imported() {
    const internal = this.#internal;
    return constructModuleExportName(internal.$pos + 8, internal.$ast);
  }

  get local() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 64, internal.$ast);
  }

  get importKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.$pos + 96, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ImportSpecifier',
      start: this.start,
      end: this.end,
      imported: this.imported,
      local: this.local,
      importKind: this.importKind,
    };
  }
}

class ImportDefaultSpecifier {
  type = 'ImportDefaultSpecifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get local() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ImportDefaultSpecifier',
      start: this.start,
      end: this.end,
      local: this.local,
    };
  }
}

class ImportNamespaceSpecifier {
  type = 'ImportNamespaceSpecifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get local() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ImportNamespaceSpecifier',
      start: this.start,
      end: this.end,
      local: this.local,
    };
  }
}

class WithClause {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, attributes: void 0 };
  }

  get attributes() {
    const internal = this.#internal,
      node = internal.attributes;
    if (node !== void 0) return node;
    return internal.attributes = constructVecImportAttribute(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      attributes: this.attributes,
    };
  }
}

class ImportAttribute {
  type = 'ImportAttribute';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructImportAttributeKey(internal.$pos + 8, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return new StringLiteral(internal.$pos + 64, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ImportAttribute',
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
    };
  }
}

function constructImportAttributeKey(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new IdentifierName(pos + 8, ast);
    case 1:
      return new StringLiteral(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportAttributeKey`);
  }
}

class ExportNamedDeclaration {
  type = 'ExportNamedDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, specifiers: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get declaration() {
    const internal = this.#internal;
    return constructOptionDeclaration(internal.$pos + 8, internal.$ast);
  }

  get specifiers() {
    const internal = this.#internal,
      node = internal.specifiers;
    if (node !== void 0) return node;
    return internal.specifiers = constructVecExportSpecifier(internal.$pos + 24, internal.$ast);
  }

  get source() {
    const internal = this.#internal;
    return constructOptionStringLiteral(internal.$pos + 48, internal.$ast);
  }

  get exportKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.$pos + 104, internal.$ast);
  }

  get attributes() {
    const internal = this.#internal;
    return constructOptionBoxWithClause(internal.$pos + 96, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ExportNamedDeclaration',
      start: this.start,
      end: this.end,
      declaration: this.declaration,
      specifiers: this.specifiers,
      source: this.source,
      exportKind: this.exportKind,
      attributes: this.attributes,
    };
  }
}

class ExportDefaultDeclaration {
  type = 'ExportDefaultDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get declaration() {
    const internal = this.#internal;
    return constructExportDefaultDeclarationKind(internal.$pos + 64, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ExportDefaultDeclaration',
      start: this.start,
      end: this.end,
      declaration: this.declaration,
    };
  }
}

class ExportAllDeclaration {
  type = 'ExportAllDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get exported() {
    const internal = this.#internal;
    return constructOptionModuleExportName(internal.$pos + 8, internal.$ast);
  }

  get source() {
    const internal = this.#internal;
    return new StringLiteral(internal.$pos + 64, internal.$ast);
  }

  get attributes() {
    const internal = this.#internal;
    return constructOptionBoxWithClause(internal.$pos + 112, internal.$ast);
  }

  get exportKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.$pos + 120, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ExportAllDeclaration',
      start: this.start,
      end: this.end,
      exported: this.exported,
      source: this.source,
      attributes: this.attributes,
      exportKind: this.exportKind,
    };
  }
}

class ExportSpecifier {
  type = 'ExportSpecifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get local() {
    const internal = this.#internal;
    return constructModuleExportName(internal.$pos + 8, internal.$ast);
  }

  get exported() {
    const internal = this.#internal;
    return constructModuleExportName(internal.$pos + 64, internal.$ast);
  }

  get exportKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.$pos + 120, internal.$ast);
  }

  toJSON() {
    return {
      type: 'ExportSpecifier',
      start: this.start,
      end: this.end,
      local: this.local,
      exported: this.exported,
      exportKind: this.exportKind,
    };
  }
}

function constructExportDefaultDeclarationKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    case 64:
      return constructBoxFunction(pos + 8, ast);
    case 65:
      return constructBoxClass(pos + 8, ast);
    case 66:
      return constructBoxTSInterfaceDeclaration(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ExportDefaultDeclarationKind`);
  }
}

function constructModuleExportName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new IdentifierName(pos + 8, ast);
    case 1:
      return new IdentifierReference(pos + 8, ast);
    case 2:
      return new StringLiteral(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ModuleExportName`);
  }
}

class V8IntrinsicExpression {
  type = 'V8IntrinsicExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, arguments: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 8, internal.$ast);
  }

  get arguments() {
    const internal = this.#internal,
      node = internal.arguments;
    if (node !== void 0) return node;
    return internal.arguments = constructVecArgument(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'V8IntrinsicExpression',
      start: this.start,
      end: this.end,
      name: this.name,
      arguments: this.arguments,
    };
  }
}

class BooleanLiteral {
  type = 'BooleanLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BooleanLiteral',
      start: this.start,
      end: this.end,
      value: this.value,
    };
  }
}

class NullLiteral {
  type = 'NullLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'NullLiteral',
      start: this.start,
      end: this.end,
    };
  }
}

class NumericLiteral {
  type = 'NumericLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, raw: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructF64(internal.$pos + 8, internal.$ast);
  }

  get raw() {
    const internal = this.#internal,
      node = internal.raw;
    if (node !== void 0) return node;
    return internal.raw = constructOptionStr(internal.$pos + 16, internal.$ast);
  }

  toJSON() {
    return {
      type: 'NumericLiteral',
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }
}

class StringLiteral {
  type = 'StringLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, value: void 0, raw: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal,
      node = internal.value;
    if (node !== void 0) return node;
    return internal.value = constructStr(internal.$pos + 8, internal.$ast);
  }

  get raw() {
    const internal = this.#internal,
      node = internal.raw;
    if (node !== void 0) return node;
    return internal.raw = constructOptionStr(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'StringLiteral',
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }
}

class BigIntLiteral {
  type = 'BigIntLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, value: void 0, raw: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal,
      node = internal.value;
    if (node !== void 0) return node;
    return internal.value = constructStr(internal.$pos + 8, internal.$ast);
  }

  get raw() {
    const internal = this.#internal,
      node = internal.raw;
    if (node !== void 0) return node;
    return internal.raw = constructOptionStr(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'BigIntLiteral',
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }
}

class RegExpLiteral {
  type = 'RegExpLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, raw: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get regex() {
    const internal = this.#internal;
    return new RegExp(internal.$pos + 8, internal.$ast);
  }

  get raw() {
    const internal = this.#internal,
      node = internal.raw;
    if (node !== void 0) return node;
    return internal.raw = constructOptionStr(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'RegExpLiteral',
      start: this.start,
      end: this.end,
      regex: this.regex,
      raw: this.raw,
    };
  }
}

class RegExp {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get pattern() {
    const internal = this.#internal;
    return new RegExpPattern(internal.$pos, internal.$ast);
  }

  get flags() {
    const internal = this.#internal;
    return new RegExpFlags(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      pattern: this.pattern,
      flags: this.flags,
    };
  }
}

class RegExpPattern {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, pattern: void 0 };
  }

  get pattern() {
    const internal = this.#internal,
      node = internal.pattern;
    if (node !== void 0) return node;
    return internal.pattern = constructStr(internal.$pos, internal.$ast);
  }

  toJSON() {
    return {
      pattern: this.pattern,
    };
  }
}

class RegExpFlags {
  type = 'RegExpFlags';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get 0() {
    const internal = this.#internal;
    return constructU8(internal.$pos, internal.$ast);
  }

  toJSON() {
    return {
      type: 'RegExpFlags',
    };
  }
}

class JSXElement {
  type = 'JSXElement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, children: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get openingElement() {
    const internal = this.#internal;
    return constructBoxJSXOpeningElement(internal.$pos + 8, internal.$ast);
  }

  get children() {
    const internal = this.#internal,
      node = internal.children;
    if (node !== void 0) return node;
    return internal.children = constructVecJSXChild(internal.$pos + 16, internal.$ast);
  }

  get closingElement() {
    const internal = this.#internal;
    return constructOptionBoxJSXClosingElement(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXElement',
      start: this.start,
      end: this.end,
      openingElement: this.openingElement,
      children: this.children,
      closingElement: this.closingElement,
    };
  }
}

class JSXOpeningElement {
  type = 'JSXOpeningElement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, attributes: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal;
    return constructJSXElementName(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  get attributes() {
    const internal = this.#internal,
      node = internal.attributes;
    if (node !== void 0) return node;
    return internal.attributes = constructVecJSXAttributeItem(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXOpeningElement',
      start: this.start,
      end: this.end,
      name: this.name,
      typeArguments: this.typeArguments,
      attributes: this.attributes,
    };
  }
}

class JSXClosingElement {
  type = 'JSXClosingElement';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal;
    return constructJSXElementName(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXClosingElement',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

class JSXFragment {
  type = 'JSXFragment';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, children: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get openingFragment() {
    const internal = this.#internal;
    return new JSXOpeningFragment(internal.$pos + 8, internal.$ast);
  }

  get children() {
    const internal = this.#internal,
      node = internal.children;
    if (node !== void 0) return node;
    return internal.children = constructVecJSXChild(internal.$pos + 16, internal.$ast);
  }

  get closingFragment() {
    const internal = this.#internal;
    return new JSXClosingFragment(internal.$pos + 40, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXFragment',
      start: this.start,
      end: this.end,
      openingFragment: this.openingFragment,
      children: this.children,
      closingFragment: this.closingFragment,
    };
  }
}

class JSXOpeningFragment {
  type = 'JSXOpeningFragment';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXOpeningFragment',
      start: this.start,
      end: this.end,
    };
  }
}

class JSXClosingFragment {
  type = 'JSXClosingFragment';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXClosingFragment',
      start: this.start,
      end: this.end,
    };
  }
}

function constructJSXElementName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxJSXIdentifier(pos + 8, ast);
    case 1:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 2:
      return constructBoxJSXNamespacedName(pos + 8, ast);
    case 3:
      return constructBoxJSXMemberExpression(pos + 8, ast);
    case 4:
      return constructBoxThisExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXElementName`);
  }
}

class JSXNamespacedName {
  type = 'JSXNamespacedName';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get namespace() {
    const internal = this.#internal;
    return new JSXIdentifier(internal.$pos + 8, internal.$ast);
  }

  get name() {
    const internal = this.#internal;
    return new JSXIdentifier(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXNamespacedName',
      start: this.start,
      end: this.end,
      namespace: this.namespace,
      name: this.name,
    };
  }
}

class JSXMemberExpression {
  type = 'JSXMemberExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get object() {
    const internal = this.#internal;
    return constructJSXMemberExpressionObject(internal.$pos + 8, internal.$ast);
  }

  get property() {
    const internal = this.#internal;
    return new JSXIdentifier(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXMemberExpression',
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
    };
  }
}

function constructJSXMemberExpressionObject(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxJSXMemberExpression(pos + 8, ast);
    case 2:
      return constructBoxThisExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXMemberExpressionObject`);
  }
}

class JSXExpressionContainer {
  type = 'JSXExpressionContainer';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructJSXExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXExpressionContainer',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

function constructJSXExpression(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNullLiteral(pos + 8, ast);
    case 2:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 3:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 4:
      return constructBoxRegExpLiteral(pos + 8, ast);
    case 5:
      return constructBoxStringLiteral(pos + 8, ast);
    case 6:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 7:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 8:
      return constructBoxMetaProperty(pos + 8, ast);
    case 9:
      return constructBoxSuper(pos + 8, ast);
    case 10:
      return constructBoxArrayExpression(pos + 8, ast);
    case 11:
      return constructBoxArrowFunctionExpression(pos + 8, ast);
    case 12:
      return constructBoxAssignmentExpression(pos + 8, ast);
    case 13:
      return constructBoxAwaitExpression(pos + 8, ast);
    case 14:
      return constructBoxBinaryExpression(pos + 8, ast);
    case 15:
      return constructBoxCallExpression(pos + 8, ast);
    case 16:
      return constructBoxChainExpression(pos + 8, ast);
    case 17:
      return constructBoxClass(pos + 8, ast);
    case 18:
      return constructBoxConditionalExpression(pos + 8, ast);
    case 19:
      return constructBoxFunction(pos + 8, ast);
    case 20:
      return constructBoxImportExpression(pos + 8, ast);
    case 21:
      return constructBoxLogicalExpression(pos + 8, ast);
    case 22:
      return constructBoxNewExpression(pos + 8, ast);
    case 23:
      return constructBoxObjectExpression(pos + 8, ast);
    case 24:
      return constructBoxParenthesizedExpression(pos + 8, ast);
    case 25:
      return constructBoxSequenceExpression(pos + 8, ast);
    case 26:
      return constructBoxTaggedTemplateExpression(pos + 8, ast);
    case 27:
      return constructBoxThisExpression(pos + 8, ast);
    case 28:
      return constructBoxUnaryExpression(pos + 8, ast);
    case 29:
      return constructBoxUpdateExpression(pos + 8, ast);
    case 30:
      return constructBoxYieldExpression(pos + 8, ast);
    case 31:
      return constructBoxPrivateInExpression(pos + 8, ast);
    case 32:
      return constructBoxJSXElement(pos + 8, ast);
    case 33:
      return constructBoxJSXFragment(pos + 8, ast);
    case 34:
      return constructBoxTSAsExpression(pos + 8, ast);
    case 35:
      return constructBoxTSSatisfiesExpression(pos + 8, ast);
    case 36:
      return constructBoxTSTypeAssertion(pos + 8, ast);
    case 37:
      return constructBoxTSNonNullExpression(pos + 8, ast);
    case 38:
      return constructBoxTSInstantiationExpression(pos + 8, ast);
    case 39:
      return constructBoxV8IntrinsicExpression(pos + 8, ast);
    case 48:
      return constructBoxComputedMemberExpression(pos + 8, ast);
    case 49:
      return constructBoxStaticMemberExpression(pos + 8, ast);
    case 50:
      return constructBoxPrivateFieldExpression(pos + 8, ast);
    case 64:
      return new JSXEmptyExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXExpression`);
  }
}

class JSXEmptyExpression {
  type = 'JSXEmptyExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXEmptyExpression',
      start: this.start,
      end: this.end,
    };
  }
}

function constructJSXAttributeItem(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxJSXAttribute(pos + 8, ast);
    case 1:
      return constructBoxJSXSpreadAttribute(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeItem`);
  }
}

class JSXAttribute {
  type = 'JSXAttribute';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal;
    return constructJSXAttributeName(internal.$pos + 8, internal.$ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionJSXAttributeValue(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXAttribute',
      start: this.start,
      end: this.end,
      name: this.name,
      value: this.value,
    };
  }
}

class JSXSpreadAttribute {
  type = 'JSXSpreadAttribute';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXSpreadAttribute',
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }
}

function constructJSXAttributeName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxJSXIdentifier(pos + 8, ast);
    case 1:
      return constructBoxJSXNamespacedName(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeName`);
  }
}

function constructJSXAttributeValue(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxStringLiteral(pos + 8, ast);
    case 1:
      return constructBoxJSXExpressionContainer(pos + 8, ast);
    case 2:
      return constructBoxJSXElement(pos + 8, ast);
    case 3:
      return constructBoxJSXFragment(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeValue`);
  }
}

class JSXIdentifier {
  type = 'JSXIdentifier';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXIdentifier',
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }
}

function constructJSXChild(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxJSXText(pos + 8, ast);
    case 1:
      return constructBoxJSXElement(pos + 8, ast);
    case 2:
      return constructBoxJSXFragment(pos + 8, ast);
    case 3:
      return constructBoxJSXExpressionContainer(pos + 8, ast);
    case 4:
      return constructBoxJSXSpreadChild(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXChild`);
  }
}

class JSXSpreadChild {
  type = 'JSXSpreadChild';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXSpreadChild',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

class JSXText {
  type = 'JSXText';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, value: void 0, raw: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get value() {
    const internal = this.#internal,
      node = internal.value;
    if (node !== void 0) return node;
    return internal.value = constructStr(internal.$pos + 8, internal.$ast);
  }

  get raw() {
    const internal = this.#internal,
      node = internal.raw;
    if (node !== void 0) return node;
    return internal.raw = constructOptionStr(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSXText',
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }
}

class TSThisParameter {
  type = 'TSThisParameter';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 16, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSThisParameter',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSEnumDeclaration {
  type = 'TSEnumDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return new TSEnumBody(internal.$pos + 40, internal.$ast);
  }

  get const() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 76, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 77, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSEnumDeclaration',
      start: this.start,
      end: this.end,
      id: this.id,
      body: this.body,
      const: this.const,
      declare: this.declare,
    };
  }
}

class TSEnumBody {
  type = 'TSEnumBody';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, members: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get members() {
    const internal = this.#internal,
      node = internal.members;
    if (node !== void 0) return node;
    return internal.members = constructVecTSEnumMember(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSEnumBody',
      start: this.start,
      end: this.end,
      members: this.members,
    };
  }
}

class TSEnumMember {
  type = 'TSEnumMember';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return constructTSEnumMemberName(internal.$pos + 8, internal.$ast);
  }

  get initializer() {
    const internal = this.#internal;
    return constructOptionExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSEnumMember',
      start: this.start,
      end: this.end,
      id: this.id,
      initializer: this.initializer,
    };
  }
}

function constructTSEnumMemberName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierName(pos + 8, ast);
    case 1:
      return constructBoxStringLiteral(pos + 8, ast);
    case 2:
      return constructBoxStringLiteral(pos + 8, ast);
    case 3:
      return constructBoxTemplateLiteral(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSEnumMemberName`);
  }
}

class TSTypeAnnotation {
  type = 'TSTypeAnnotation';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeAnnotation',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSLiteralType {
  type = 'TSLiteralType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get literal() {
    const internal = this.#internal;
    return constructTSLiteral(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSLiteralType',
      start: this.start,
      end: this.end,
      literal: this.literal,
    };
  }
}

function constructTSLiteral(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxBooleanLiteral(pos + 8, ast);
    case 1:
      return constructBoxNumericLiteral(pos + 8, ast);
    case 2:
      return constructBoxBigIntLiteral(pos + 8, ast);
    case 3:
      return constructBoxStringLiteral(pos + 8, ast);
    case 4:
      return constructBoxTemplateLiteral(pos + 8, ast);
    case 5:
      return constructBoxUnaryExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSLiteral`);
  }
}

function constructTSType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxTSAnyKeyword(pos + 8, ast);
    case 1:
      return constructBoxTSBigIntKeyword(pos + 8, ast);
    case 2:
      return constructBoxTSBooleanKeyword(pos + 8, ast);
    case 3:
      return constructBoxTSIntrinsicKeyword(pos + 8, ast);
    case 4:
      return constructBoxTSNeverKeyword(pos + 8, ast);
    case 5:
      return constructBoxTSNullKeyword(pos + 8, ast);
    case 6:
      return constructBoxTSNumberKeyword(pos + 8, ast);
    case 7:
      return constructBoxTSObjectKeyword(pos + 8, ast);
    case 8:
      return constructBoxTSStringKeyword(pos + 8, ast);
    case 9:
      return constructBoxTSSymbolKeyword(pos + 8, ast);
    case 10:
      return constructBoxTSThisType(pos + 8, ast);
    case 11:
      return constructBoxTSUndefinedKeyword(pos + 8, ast);
    case 12:
      return constructBoxTSUnknownKeyword(pos + 8, ast);
    case 13:
      return constructBoxTSVoidKeyword(pos + 8, ast);
    case 14:
      return constructBoxTSArrayType(pos + 8, ast);
    case 15:
      return constructBoxTSConditionalType(pos + 8, ast);
    case 16:
      return constructBoxTSConstructorType(pos + 8, ast);
    case 17:
      return constructBoxTSFunctionType(pos + 8, ast);
    case 18:
      return constructBoxTSImportType(pos + 8, ast);
    case 19:
      return constructBoxTSIndexedAccessType(pos + 8, ast);
    case 20:
      return constructBoxTSInferType(pos + 8, ast);
    case 21:
      return constructBoxTSIntersectionType(pos + 8, ast);
    case 22:
      return constructBoxTSLiteralType(pos + 8, ast);
    case 23:
      return constructBoxTSMappedType(pos + 8, ast);
    case 24:
      return constructBoxTSNamedTupleMember(pos + 8, ast);
    case 26:
      return constructBoxTSTemplateLiteralType(pos + 8, ast);
    case 27:
      return constructBoxTSTupleType(pos + 8, ast);
    case 28:
      return constructBoxTSTypeLiteral(pos + 8, ast);
    case 29:
      return constructBoxTSTypeOperator(pos + 8, ast);
    case 30:
      return constructBoxTSTypePredicate(pos + 8, ast);
    case 31:
      return constructBoxTSTypeQuery(pos + 8, ast);
    case 32:
      return constructBoxTSTypeReference(pos + 8, ast);
    case 33:
      return constructBoxTSUnionType(pos + 8, ast);
    case 34:
      return constructBoxTSParenthesizedType(pos + 8, ast);
    case 35:
      return constructBoxJSDocNullableType(pos + 8, ast);
    case 36:
      return constructBoxJSDocNonNullableType(pos + 8, ast);
    case 37:
      return constructBoxJSDocUnknownType(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSType`);
  }
}

class TSConditionalType {
  type = 'TSConditionalType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get checkType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  get extendsType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 24, internal.$ast);
  }

  get trueType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 40, internal.$ast);
  }

  get falseType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 56, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSConditionalType',
      start: this.start,
      end: this.end,
      checkType: this.checkType,
      extendsType: this.extendsType,
      trueType: this.trueType,
      falseType: this.falseType,
    };
  }
}

class TSUnionType {
  type = 'TSUnionType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, types: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get types() {
    const internal = this.#internal,
      node = internal.types;
    if (node !== void 0) return node;
    return internal.types = constructVecTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSUnionType',
      start: this.start,
      end: this.end,
      types: this.types,
    };
  }
}

class TSIntersectionType {
  type = 'TSIntersectionType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, types: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get types() {
    const internal = this.#internal,
      node = internal.types;
    if (node !== void 0) return node;
    return internal.types = constructVecTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSIntersectionType',
      start: this.start,
      end: this.end,
      types: this.types,
    };
  }
}

class TSParenthesizedType {
  type = 'TSParenthesizedType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSParenthesizedType',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSTypeOperator {
  type = 'TSTypeOperator';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructTSTypeOperatorOperator(internal.$pos + 24, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeOperator',
      start: this.start,
      end: this.end,
      operator: this.operator,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

function constructTSTypeOperatorOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'keyof';
    case 1:
      return 'unique';
    case 2:
      return 'readonly';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeOperatorOperator`);
  }
}

class TSArrayType {
  type = 'TSArrayType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get elementType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSArrayType',
      start: this.start,
      end: this.end,
      elementType: this.elementType,
    };
  }
}

class TSIndexedAccessType {
  type = 'TSIndexedAccessType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get objectType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  get indexType() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSIndexedAccessType',
      start: this.start,
      end: this.end,
      objectType: this.objectType,
      indexType: this.indexType,
    };
  }
}

class TSTupleType {
  type = 'TSTupleType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, elementTypes: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get elementTypes() {
    const internal = this.#internal,
      node = internal.elementTypes;
    if (node !== void 0) return node;
    return internal.elementTypes = constructVecTSTupleElement(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTupleType',
      start: this.start,
      end: this.end,
      elementTypes: this.elementTypes,
    };
  }
}

class TSNamedTupleMember {
  type = 'TSNamedTupleMember';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get label() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 8, internal.$ast);
  }

  get elementType() {
    const internal = this.#internal;
    return constructTSTupleElement(internal.$pos + 32, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 48, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSNamedTupleMember',
      start: this.start,
      end: this.end,
      label: this.label,
      elementType: this.elementType,
      optional: this.optional,
    };
  }
}

class TSOptionalType {
  type = 'TSOptionalType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSOptionalType',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSRestType {
  type = 'TSRestType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSRestType',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

function constructTSTupleElement(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxTSAnyKeyword(pos + 8, ast);
    case 1:
      return constructBoxTSBigIntKeyword(pos + 8, ast);
    case 2:
      return constructBoxTSBooleanKeyword(pos + 8, ast);
    case 3:
      return constructBoxTSIntrinsicKeyword(pos + 8, ast);
    case 4:
      return constructBoxTSNeverKeyword(pos + 8, ast);
    case 5:
      return constructBoxTSNullKeyword(pos + 8, ast);
    case 6:
      return constructBoxTSNumberKeyword(pos + 8, ast);
    case 7:
      return constructBoxTSObjectKeyword(pos + 8, ast);
    case 8:
      return constructBoxTSStringKeyword(pos + 8, ast);
    case 9:
      return constructBoxTSSymbolKeyword(pos + 8, ast);
    case 10:
      return constructBoxTSThisType(pos + 8, ast);
    case 11:
      return constructBoxTSUndefinedKeyword(pos + 8, ast);
    case 12:
      return constructBoxTSUnknownKeyword(pos + 8, ast);
    case 13:
      return constructBoxTSVoidKeyword(pos + 8, ast);
    case 14:
      return constructBoxTSArrayType(pos + 8, ast);
    case 15:
      return constructBoxTSConditionalType(pos + 8, ast);
    case 16:
      return constructBoxTSConstructorType(pos + 8, ast);
    case 17:
      return constructBoxTSFunctionType(pos + 8, ast);
    case 18:
      return constructBoxTSImportType(pos + 8, ast);
    case 19:
      return constructBoxTSIndexedAccessType(pos + 8, ast);
    case 20:
      return constructBoxTSInferType(pos + 8, ast);
    case 21:
      return constructBoxTSIntersectionType(pos + 8, ast);
    case 22:
      return constructBoxTSLiteralType(pos + 8, ast);
    case 23:
      return constructBoxTSMappedType(pos + 8, ast);
    case 24:
      return constructBoxTSNamedTupleMember(pos + 8, ast);
    case 26:
      return constructBoxTSTemplateLiteralType(pos + 8, ast);
    case 27:
      return constructBoxTSTupleType(pos + 8, ast);
    case 28:
      return constructBoxTSTypeLiteral(pos + 8, ast);
    case 29:
      return constructBoxTSTypeOperator(pos + 8, ast);
    case 30:
      return constructBoxTSTypePredicate(pos + 8, ast);
    case 31:
      return constructBoxTSTypeQuery(pos + 8, ast);
    case 32:
      return constructBoxTSTypeReference(pos + 8, ast);
    case 33:
      return constructBoxTSUnionType(pos + 8, ast);
    case 34:
      return constructBoxTSParenthesizedType(pos + 8, ast);
    case 35:
      return constructBoxJSDocNullableType(pos + 8, ast);
    case 36:
      return constructBoxJSDocNonNullableType(pos + 8, ast);
    case 37:
      return constructBoxJSDocUnknownType(pos + 8, ast);
    case 64:
      return constructBoxTSOptionalType(pos + 8, ast);
    case 65:
      return constructBoxTSRestType(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTupleElement`);
  }
}

class TSAnyKeyword {
  type = 'TSAnyKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSAnyKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSStringKeyword {
  type = 'TSStringKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSStringKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSBooleanKeyword {
  type = 'TSBooleanKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSBooleanKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSNumberKeyword {
  type = 'TSNumberKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSNumberKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSNeverKeyword {
  type = 'TSNeverKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSNeverKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSIntrinsicKeyword {
  type = 'TSIntrinsicKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSIntrinsicKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSUnknownKeyword {
  type = 'TSUnknownKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSUnknownKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSNullKeyword {
  type = 'TSNullKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSNullKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSUndefinedKeyword {
  type = 'TSUndefinedKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSUndefinedKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSVoidKeyword {
  type = 'TSVoidKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSVoidKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSSymbolKeyword {
  type = 'TSSymbolKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSSymbolKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSThisType {
  type = 'TSThisType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSThisType',
      start: this.start,
      end: this.end,
    };
  }
}

class TSObjectKeyword {
  type = 'TSObjectKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSObjectKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSBigIntKeyword {
  type = 'TSBigIntKeyword';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSBigIntKeyword',
      start: this.start,
      end: this.end,
    };
  }
}

class TSTypeReference {
  type = 'TSTypeReference';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeName() {
    const internal = this.#internal;
    return constructTSTypeName(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeReference',
      start: this.start,
      end: this.end,
      typeName: this.typeName,
      typeArguments: this.typeArguments,
    };
  }
}

function constructTSTypeName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSQualifiedName(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeName`);
  }
}

class TSQualifiedName {
  type = 'TSQualifiedName';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get left() {
    const internal = this.#internal;
    return constructTSTypeName(internal.$pos + 8, internal.$ast);
  }

  get right() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSQualifiedName',
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }
}

class TSTypeParameterInstantiation {
  type = 'TSTypeParameterInstantiation';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, params: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get params() {
    const internal = this.#internal,
      node = internal.params;
    if (node !== void 0) return node;
    return internal.params = constructVecTSType(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeParameterInstantiation',
      start: this.start,
      end: this.end,
      params: this.params,
    };
  }
}

class TSTypeParameter {
  type = 'TSTypeParameter';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  get constraint() {
    const internal = this.#internal;
    return constructOptionTSType(internal.$pos + 40, internal.$ast);
  }

  get default() {
    const internal = this.#internal;
    return constructOptionTSType(internal.$pos + 56, internal.$ast);
  }

  get in() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 72, internal.$ast);
  }

  get out() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 73, internal.$ast);
  }

  get const() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 74, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeParameter',
      start: this.start,
      end: this.end,
      name: this.name,
      constraint: this.constraint,
      default: this.default,
      in: this.in,
      out: this.out,
      const: this.const,
    };
  }
}

class TSTypeParameterDeclaration {
  type = 'TSTypeParameterDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, params: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get params() {
    const internal = this.#internal,
      node = internal.params;
    if (node !== void 0) return node;
    return internal.params = constructVecTSTypeParameter(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeParameterDeclaration',
      start: this.start,
      end: this.end,
      params: this.params,
    };
  }
}

class TSTypeAliasDeclaration {
  type = 'TSTypeAliasDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 40, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 48, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 68, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeAliasDeclaration',
      start: this.start,
      end: this.end,
      id: this.id,
      typeParameters: this.typeParameters,
      typeAnnotation: this.typeAnnotation,
      declare: this.declare,
    };
  }
}

function constructTSAccessibility(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'private';
    case 1:
      return 'protected';
    case 2:
      return 'public';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSAccessibility`);
  }
}

class TSClassImplements {
  type = 'TSClassImplements';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructTSTypeName(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSClassImplements',
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeArguments: this.typeArguments,
    };
  }
}

class TSInterfaceDeclaration {
  type = 'TSInterfaceDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, extends: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 40, internal.$ast);
  }

  get extends() {
    const internal = this.#internal,
      node = internal.extends;
    if (node !== void 0) return node;
    return internal.extends = constructVecTSInterfaceHeritage(internal.$pos + 48, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructBoxTSInterfaceBody(internal.$pos + 72, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 84, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSInterfaceDeclaration',
      start: this.start,
      end: this.end,
      id: this.id,
      typeParameters: this.typeParameters,
      extends: this.extends,
      body: this.body,
      declare: this.declare,
    };
  }
}

class TSInterfaceBody {
  type = 'TSInterfaceBody';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecTSSignature(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSInterfaceBody',
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }
}

class TSPropertySignature {
  type = 'TSPropertySignature';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 32, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 33, internal.$ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 34, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 8, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSPropertySignature',
      start: this.start,
      end: this.end,
      computed: this.computed,
      optional: this.optional,
      readonly: this.readonly,
      key: this.key,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

function constructTSSignature(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxTSIndexSignature(pos + 8, ast);
    case 1:
      return constructBoxTSPropertySignature(pos + 8, ast);
    case 2:
      return constructBoxTSCallSignatureDeclaration(pos + 8, ast);
    case 3:
      return constructBoxTSConstructSignatureDeclaration(pos + 8, ast);
    case 4:
      return constructBoxTSMethodSignature(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSSignature`);
  }
}

class TSIndexSignature {
  type = 'TSIndexSignature';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, parameters: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get parameters() {
    const internal = this.#internal,
      node = internal.parameters;
    if (node !== void 0) return node;
    return internal.parameters = constructVecTSIndexSignatureName(internal.$pos + 8, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.$pos + 32, internal.$ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 40, internal.$ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 41, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSIndexSignature',
      start: this.start,
      end: this.end,
      parameters: this.parameters,
      typeAnnotation: this.typeAnnotation,
      readonly: this.readonly,
      static: this.static,
    };
  }
}

class TSCallSignatureDeclaration {
  type = 'TSCallSignatureDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 8, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 24, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSCallSignatureDeclaration',
      start: this.start,
      end: this.end,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }
}

function constructTSMethodSignatureKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'method';
    case 1:
      return 'get';
    case 2:
      return 'set';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSMethodSignatureKind`);
  }
}

class TSMethodSignature {
  type = 'TSMethodSignature';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.$pos + 8, internal.$ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 60, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 61, internal.$ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructTSMethodSignatureKind(internal.$pos + 62, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 24, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 40, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 48, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSMethodSignature',
      start: this.start,
      end: this.end,
      key: this.key,
      computed: this.computed,
      optional: this.optional,
      kind: this.kind,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }
}

class TSConstructSignatureDeclaration {
  type = 'TSConstructSignatureDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 8, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 16, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSConstructSignatureDeclaration',
      start: this.start,
      end: this.end,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }
}

class TSIndexSignatureName {
  type = 'TSIndexSignatureName';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, name: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get name() {
    const internal = this.#internal,
      node = internal.name;
    if (node !== void 0) return node;
    return internal.name = constructStr(internal.$pos + 8, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSIndexSignatureName',
      start: this.start,
      end: this.end,
      name: this.name,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSInterfaceHeritage {
  type = 'TSInterfaceHeritage';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSInterfaceHeritage',
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeArguments: this.typeArguments,
    };
  }
}

class TSTypePredicate {
  type = 'TSTypePredicate';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get parameterName() {
    const internal = this.#internal;
    return constructTSTypePredicateName(internal.$pos + 8, internal.$ast);
  }

  get asserts() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 32, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypePredicate',
      start: this.start,
      end: this.end,
      parameterName: this.parameterName,
      asserts: this.asserts,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

function constructTSTypePredicateName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierName(pos + 8, ast);
    case 1:
      return new TSThisType(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypePredicateName`);
  }
}

class TSModuleDeclaration {
  type = 'TSModuleDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return constructTSModuleDeclarationName(internal.$pos + 8, internal.$ast);
  }

  get body() {
    const internal = this.#internal;
    return constructOptionTSModuleDeclarationBody(internal.$pos + 64, internal.$ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructTSModuleDeclarationKind(internal.$pos + 84, internal.$ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 85, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSModuleDeclaration',
      start: this.start,
      end: this.end,
      id: this.id,
      body: this.body,
      kind: this.kind,
      declare: this.declare,
    };
  }
}

function constructTSModuleDeclarationKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'global';
    case 1:
      return 'module';
    case 2:
      return 'namespace';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationKind`);
  }
}

function constructTSModuleDeclarationName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new BindingIdentifier(pos + 8, ast);
    case 1:
      return new StringLiteral(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationName`);
  }
}

function constructTSModuleDeclarationBody(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxTSModuleDeclaration(pos + 8, ast);
    case 1:
      return constructBoxTSModuleBlock(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationBody`);
  }
}

class TSModuleBlock {
  type = 'TSModuleBlock';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, body: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get body() {
    const internal = this.#internal,
      node = internal.body;
    if (node !== void 0) return node;
    return internal.body = constructVecStatement(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSModuleBlock',
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }
}

class TSTypeLiteral {
  type = 'TSTypeLiteral';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, members: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get members() {
    const internal = this.#internal,
      node = internal.members;
    if (node !== void 0) return node;
    return internal.members = constructVecTSSignature(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeLiteral',
      start: this.start,
      end: this.end,
      members: this.members,
    };
  }
}

class TSInferType {
  type = 'TSInferType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeParameter() {
    const internal = this.#internal;
    return constructBoxTSTypeParameter(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSInferType',
      start: this.start,
      end: this.end,
      typeParameter: this.typeParameter,
    };
  }
}

class TSTypeQuery {
  type = 'TSTypeQuery';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get exprName() {
    const internal = this.#internal;
    return constructTSTypeQueryExprName(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeQuery',
      start: this.start,
      end: this.end,
      exprName: this.exprName,
      typeArguments: this.typeArguments,
    };
  }
}

function constructTSTypeQueryExprName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSQualifiedName(pos + 8, ast);
    case 2:
      return constructBoxTSImportType(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeQueryExprName`);
  }
}

class TSImportType {
  type = 'TSImportType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  get options() {
    const internal = this.#internal;
    return constructOptionBoxObjectExpression(internal.$pos + 24, internal.$ast);
  }

  get qualifier() {
    const internal = this.#internal;
    return constructOptionTSTypeName(internal.$pos + 32, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.$pos + 48, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSImportType',
      start: this.start,
      end: this.end,
      argument: this.argument,
      options: this.options,
      qualifier: this.qualifier,
      typeArguments: this.typeArguments,
    };
  }
}

class TSFunctionType {
  type = 'TSFunctionType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 8, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 24, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSFunctionType',
      start: this.start,
      end: this.end,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }
}

class TSConstructorType {
  type = 'TSConstructorType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get abstract() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 32, internal.$ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.$pos + 8, internal.$ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.$pos + 16, internal.$ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSConstructorType',
      start: this.start,
      end: this.end,
      abstract: this.abstract,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }
}

class TSMappedType {
  type = 'TSMappedType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get nameType() {
    const internal = this.#internal;
    return constructOptionTSType(internal.$pos + 16, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionTSType(internal.$pos + 32, internal.$ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructOptionTSMappedTypeModifierOperator(internal.$pos + 52, internal.$ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructOptionTSMappedTypeModifierOperator(internal.$pos + 53, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSMappedType',
      start: this.start,
      end: this.end,
      nameType: this.nameType,
      typeAnnotation: this.typeAnnotation,
      optional: this.optional,
      readonly: this.readonly,
    };
  }
}

function constructTSMappedTypeModifierOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'true';
    case 1:
      return '+';
    case 2:
      return '-';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSMappedTypeModifierOperator`);
  }
}

class TSTemplateLiteralType {
  type = 'TSTemplateLiteralType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, quasis: void 0, types: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get quasis() {
    const internal = this.#internal,
      node = internal.quasis;
    if (node !== void 0) return node;
    return internal.quasis = constructVecTemplateElement(internal.$pos + 8, internal.$ast);
  }

  get types() {
    const internal = this.#internal,
      node = internal.types;
    if (node !== void 0) return node;
    return internal.types = constructVecTSType(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTemplateLiteralType',
      start: this.start,
      end: this.end,
      quasis: this.quasis,
      types: this.types,
    };
  }
}

class TSAsExpression {
  type = 'TSAsExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSAsExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSSatisfiesExpression {
  type = 'TSSatisfiesExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSSatisfiesExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeAnnotation: this.typeAnnotation,
    };
  }
}

class TSTypeAssertion {
  type = 'TSTypeAssertion';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSTypeAssertion',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
      expression: this.expression,
    };
  }
}

class TSImportEqualsDeclaration {
  type = 'TSImportEqualsDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.$pos + 8, internal.$ast);
  }

  get moduleReference() {
    const internal = this.#internal;
    return constructTSModuleReference(internal.$pos + 40, internal.$ast);
  }

  get importKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.$pos + 56, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSImportEqualsDeclaration',
      start: this.start,
      end: this.end,
      id: this.id,
      moduleReference: this.moduleReference,
      importKind: this.importKind,
    };
  }
}

function constructTSModuleReference(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSQualifiedName(pos + 8, ast);
    case 2:
      return constructBoxTSExternalModuleReference(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleReference`);
  }
}

class TSExternalModuleReference {
  type = 'TSExternalModuleReference';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return new StringLiteral(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSExternalModuleReference',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

class TSNonNullExpression {
  type = 'TSNonNullExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSNonNullExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

class Decorator {
  type = 'Decorator';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'Decorator',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

class TSExportAssignment {
  type = 'TSExportAssignment';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSExportAssignment',
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }
}

class TSNamespaceExportDeclaration {
  type = 'TSNamespaceExportDeclaration';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get id() {
    const internal = this.#internal;
    return new IdentifierName(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSNamespaceExportDeclaration',
      start: this.start,
      end: this.end,
      id: this.id,
    };
  }
}

class TSInstantiationExpression {
  type = 'TSInstantiationExpression';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.$pos + 8, internal.$ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructBoxTSTypeParameterInstantiation(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'TSInstantiationExpression',
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeArguments: this.typeArguments,
    };
  }
}

function constructImportOrExportKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'value';
    case 1:
      return 'type';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportOrExportKind`);
  }
}

class JSDocNullableType {
  type = 'JSDocNullableType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  get postfix() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSDocNullableType',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
      postfix: this.postfix,
    };
  }
}

class JSDocNonNullableType {
  type = 'JSDocNonNullableType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.$pos + 8, internal.$ast);
  }

  get postfix() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 24, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSDocNonNullableType',
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
      postfix: this.postfix,
    };
  }
}

class JSDocUnknownType {
  type = 'JSDocUnknownType';
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      type: 'JSDocUnknownType',
      start: this.start,
      end: this.end,
    };
  }
}

function constructCommentKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'Line';
    case 1:
      return 'Block';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for CommentKind`);
  }
}

class Comment {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get type() {
    const internal = this.#internal;
    return constructCommentKind(internal.$pos + 12, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
    };
  }
}

class NameSpan {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, value: void 0 };
  }

  get value() {
    const internal = this.#internal,
      node = internal.value;
    if (node !== void 0) return node;
    return internal.value = constructStr(internal.$pos + 8, internal.$ast);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      value: this.value,
      start: this.start,
      end: this.end,
    };
  }
}

class ImportEntry {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get importName() {
    const internal = this.#internal;
    return constructImportImportName(internal.$pos + 32, internal.$ast);
  }

  get localName() {
    const internal = this.#internal;
    return new NameSpan(internal.$pos + 64, internal.$ast);
  }

  get isType() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 88, internal.$ast);
  }

  toJSON() {
    return {
      importName: this.importName,
      localName: this.localName,
      isType: this.isType,
    };
  }
}

function constructImportImportName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new NameSpan(pos + 8, ast);
    case 1:
      return 'namespaceObject';
    case 2:
      return new Span(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportImportName`);
  }
}

class ExportEntry {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get moduleRequest() {
    const internal = this.#internal;
    return constructOptionNameSpan(internal.$pos + 16, internal.$ast);
  }

  get importName() {
    const internal = this.#internal;
    return constructExportImportName(internal.$pos + 40, internal.$ast);
  }

  get exportName() {
    const internal = this.#internal;
    return constructExportExportName(internal.$pos + 72, internal.$ast);
  }

  get localName() {
    const internal = this.#internal;
    return constructExportLocalName(internal.$pos + 104, internal.$ast);
  }

  get isType() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 136, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      moduleRequest: this.moduleRequest,
      importName: this.importName,
      exportName: this.exportName,
      localName: this.localName,
      isType: this.isType,
    };
  }
}

function constructExportImportName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new NameSpan(pos + 8, ast);
    case 1:
      return 'all';
    case 2:
      return 'allButDefault';
    case 3:
      return 'null';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ExportImportName`);
  }
}

function constructExportExportName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new NameSpan(pos + 8, ast);
    case 1:
      return new Span(pos + 8, ast);
    case 2:
      return 'null';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ExportExportName`);
  }
}

function constructExportLocalName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new NameSpan(pos + 8, ast);
    case 1:
      return new NameSpan(pos + 8, ast);
    case 2:
      return 'null';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ExportLocalName`);
  }
}

class DynamicImport {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get moduleRequest() {
    const internal = this.#internal;
    return new Span(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      moduleRequest: this.moduleRequest,
    };
  }
}

function constructAssignmentOperator(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentOperator`);
  }
}

function constructBinaryOperator(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for BinaryOperator`);
  }
}

function constructLogicalOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return '||';
    case 1:
      return '&&';
    case 2:
      return '??';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for LogicalOperator`);
  }
}

function constructUnaryOperator(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for UnaryOperator`);
  }
}

function constructUpdateOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return '++';
    case 1:
      return '--';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for UpdateOperator`);
  }
}

class Span {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
    };
  }
}

class SourceType {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast };
  }

  get sourceType() {
    const internal = this.#internal;
    return constructModuleKind(internal.$pos + 1, internal.$ast);
  }

  toJSON() {
    return {
      sourceType: this.sourceType,
    };
  }
}

function constructModuleKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'script';
    case 1:
      return 'module';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ModuleKind`);
  }
}

class RawTransferData {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, comments: void 0, errors: void 0 };
  }

  get program() {
    const internal = this.#internal;
    return new Program(internal.$pos, internal.$ast);
  }

  get comments() {
    const internal = this.#internal,
      node = internal.comments;
    if (node !== void 0) return node;
    return internal.comments = constructVecComment(internal.$pos + 128, internal.$ast);
  }

  get module() {
    const internal = this.#internal;
    return new EcmaScriptModule(internal.$pos + 152, internal.$ast);
  }

  get errors() {
    const internal = this.#internal,
      node = internal.errors;
    if (node !== void 0) return node;
    return internal.errors = constructVecError(internal.$pos + 256, internal.$ast);
  }

  toJSON() {
    return {
      program: this.program,
      comments: this.comments,
      module: this.module,
      errors: this.errors,
    };
  }
}

class Error {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, message: void 0, labels: void 0, helpMessage: void 0, codeframe: void 0 };
  }

  get severity() {
    const internal = this.#internal;
    return constructErrorSeverity(internal.$pos + 72, internal.$ast);
  }

  get message() {
    const internal = this.#internal,
      node = internal.message;
    if (node !== void 0) return node;
    return internal.message = constructStr(internal.$pos, internal.$ast);
  }

  get labels() {
    const internal = this.#internal,
      node = internal.labels;
    if (node !== void 0) return node;
    return internal.labels = constructVecErrorLabel(internal.$pos + 16, internal.$ast);
  }

  get helpMessage() {
    const internal = this.#internal,
      node = internal.helpMessage;
    if (node !== void 0) return node;
    return internal.helpMessage = constructOptionStr(internal.$pos + 40, internal.$ast);
  }

  get codeframe() {
    const internal = this.#internal,
      node = internal.codeframe;
    if (node !== void 0) return node;
    return internal.codeframe = constructStr(internal.$pos + 56, internal.$ast);
  }

  toJSON() {
    return {
      severity: this.severity,
      message: this.message,
      labels: this.labels,
      helpMessage: this.helpMessage,
      codeframe: this.codeframe,
    };
  }
}

function constructErrorSeverity(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return 'Error';
    case 1:
      return 'Warning';
    case 2:
      return 'Advice';
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ErrorSeverity`);
  }
}

class ErrorLabel {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, message: void 0 };
  }

  get message() {
    const internal = this.#internal,
      node = internal.message;
    if (node !== void 0) return node;
    return internal.message = constructOptionStr(internal.$pos + 8, internal.$ast);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  toJSON() {
    return {
      message: this.message,
      start: this.start,
      end: this.end,
    };
  }
}

class EcmaScriptModule {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = {
      $pos: pos,
      $ast: ast,
      staticImports: void 0,
      staticExports: void 0,
      dynamicImports: void 0,
      importMetas: void 0,
    };
  }

  get hasModuleSyntax() {
    const internal = this.#internal;
    return constructBool(internal.$pos + 96, internal.$ast);
  }

  get staticImports() {
    const internal = this.#internal,
      node = internal.staticImports;
    if (node !== void 0) return node;
    return internal.staticImports = constructVecStaticImport(internal.$pos, internal.$ast);
  }

  get staticExports() {
    const internal = this.#internal,
      node = internal.staticExports;
    if (node !== void 0) return node;
    return internal.staticExports = constructVecStaticExport(internal.$pos + 24, internal.$ast);
  }

  get dynamicImports() {
    const internal = this.#internal,
      node = internal.dynamicImports;
    if (node !== void 0) return node;
    return internal.dynamicImports = constructVecDynamicImport(internal.$pos + 48, internal.$ast);
  }

  get importMetas() {
    const internal = this.#internal,
      node = internal.importMetas;
    if (node !== void 0) return node;
    return internal.importMetas = constructVecSpan(internal.$pos + 72, internal.$ast);
  }

  toJSON() {
    return {
      hasModuleSyntax: this.hasModuleSyntax,
      staticImports: this.staticImports,
      staticExports: this.staticExports,
      dynamicImports: this.dynamicImports,
      importMetas: this.importMetas,
    };
  }
}

class StaticImport {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, entries: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get moduleRequest() {
    const internal = this.#internal;
    return new NameSpan(internal.$pos + 8, internal.$ast);
  }

  get entries() {
    const internal = this.#internal,
      node = internal.entries;
    if (node !== void 0) return node;
    return internal.entries = constructVecImportEntry(internal.$pos + 32, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      moduleRequest: this.moduleRequest,
      entries: this.entries,
    };
  }
}

class StaticExport {
  #internal;

  constructor(pos, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
    this.#internal = { $pos: pos, $ast: ast, entries: void 0 };
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.$pos, internal.$ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.$pos + 4, internal.$ast);
  }

  get entries() {
    const internal = this.#internal,
      node = internal.entries;
    if (node !== void 0) return node;
    return internal.entries = constructVecExportEntry(internal.$pos + 8, internal.$ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      entries: this.entries,
    };
  }
}

function constructStr(pos, ast) {
  const pos32 = pos >> 2,
    { buffer } = ast,
    { uint32 } = buffer,
    len = uint32[pos32 + 2];
  if (len === 0) return '';

  pos = uint32[pos32];
  if (ast.sourceIsAscii && pos < ast.sourceLen) return ast.sourceText.substr(pos, len);

  // Longer strings use `TextDecoder`
  // TODO: Find best switch-over point
  const end = pos + len;
  if (len > 50) return decodeStr(buffer.subarray(pos, end));

  // Shorter strings decode by hand to avoid native call
  let out = '',
    c;
  do {
    c = buffer[pos++];
    if (c < 0x80) {
      out += fromCodePoint(c);
    } else {
      out += decodeStr(buffer.subarray(pos - 1, end));
      break;
    }
  } while (pos < end);

  return out;
}

function constructVecComment(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new Comment(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructOptionHashbang(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0) return null;
  return new Hashbang(pos, ast);
}

function constructVecDirective(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new Directive(pos, ast));
    pos += 72;
  }
  return arr;
}

function constructVecStatement(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructStatement(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxBooleanLiteral(pos, ast) {
  return new BooleanLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxNullLiteral(pos, ast) {
  return new NullLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxNumericLiteral(pos, ast) {
  return new NumericLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxBigIntLiteral(pos, ast) {
  return new BigIntLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxRegExpLiteral(pos, ast) {
  return new RegExpLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxStringLiteral(pos, ast) {
  return new StringLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTemplateLiteral(pos, ast) {
  return new TemplateLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxIdentifierReference(pos, ast) {
  return new IdentifierReference(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxMetaProperty(pos, ast) {
  return new MetaProperty(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxSuper(pos, ast) {
  return new Super(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxArrayExpression(pos, ast) {
  return new ArrayExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxArrowFunctionExpression(pos, ast) {
  return new ArrowFunctionExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxAssignmentExpression(pos, ast) {
  return new AssignmentExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxAwaitExpression(pos, ast) {
  return new AwaitExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxBinaryExpression(pos, ast) {
  return new BinaryExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxCallExpression(pos, ast) {
  return new CallExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxChainExpression(pos, ast) {
  return new ChainExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxClass(pos, ast) {
  return new Class(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxConditionalExpression(pos, ast) {
  return new ConditionalExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxFunction(pos, ast) {
  return new Function(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxImportExpression(pos, ast) {
  return new ImportExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxLogicalExpression(pos, ast) {
  return new LogicalExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxNewExpression(pos, ast) {
  return new NewExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxObjectExpression(pos, ast) {
  return new ObjectExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxParenthesizedExpression(pos, ast) {
  return new ParenthesizedExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxSequenceExpression(pos, ast) {
  return new SequenceExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTaggedTemplateExpression(pos, ast) {
  return new TaggedTemplateExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxThisExpression(pos, ast) {
  return new ThisExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxUnaryExpression(pos, ast) {
  return new UnaryExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxUpdateExpression(pos, ast) {
  return new UpdateExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxYieldExpression(pos, ast) {
  return new YieldExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxPrivateInExpression(pos, ast) {
  return new PrivateInExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXElement(pos, ast) {
  return new JSXElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXFragment(pos, ast) {
  return new JSXFragment(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSAsExpression(pos, ast) {
  return new TSAsExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSSatisfiesExpression(pos, ast) {
  return new TSSatisfiesExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeAssertion(pos, ast) {
  return new TSTypeAssertion(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSNonNullExpression(pos, ast) {
  return new TSNonNullExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSInstantiationExpression(pos, ast) {
  return new TSInstantiationExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxV8IntrinsicExpression(pos, ast) {
  return new V8IntrinsicExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecArrayExpressionElement(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructArrayExpressionElement(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxSpreadElement(pos, ast) {
  return new SpreadElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecObjectPropertyKind(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructObjectPropertyKind(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxObjectProperty(pos, ast) {
  return new ObjectProperty(ast.buffer.uint32[pos >> 2], ast);
}

function constructBool(pos, ast) {
  return ast.buffer[pos] === 1;
}

function constructBoxIdentifierName(pos, ast) {
  return new IdentifierName(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxPrivateIdentifier(pos, ast) {
  return new PrivateIdentifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecTemplateElement(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new TemplateElement(pos, ast));
    pos += 48;
  }
  return arr;
}

function constructVecExpression(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructExpression(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxTSTypeParameterInstantiation(pos, ast) {
  return new TSTypeParameterInstantiation(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxTSTypeParameterInstantiation(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxTSTypeParameterInstantiation(pos, ast);
}

function constructOptionStr(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructStr(pos, ast);
}

function constructBoxComputedMemberExpression(pos, ast) {
  return new ComputedMemberExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxStaticMemberExpression(pos, ast) {
  return new StaticMemberExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxPrivateFieldExpression(pos, ast) {
  return new PrivateFieldExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecArgument(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructArgument(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxArrayAssignmentTarget(pos, ast) {
  return new ArrayAssignmentTarget(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxObjectAssignmentTarget(pos, ast) {
  return new ObjectAssignmentTarget(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionAssignmentTargetMaybeDefault(pos, ast) {
  if (ast.buffer[pos] === 51) return null;
  return constructAssignmentTargetMaybeDefault(pos, ast);
}

function constructVecOptionAssignmentTargetMaybeDefault(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructOptionAssignmentTargetMaybeDefault(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructOptionAssignmentTargetRest(pos, ast) {
  if (ast.buffer[pos + 8] === 51) return null;
  return new AssignmentTargetRest(pos, ast);
}

function constructVecAssignmentTargetProperty(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructAssignmentTargetProperty(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxAssignmentTargetWithDefault(pos, ast) {
  return new AssignmentTargetWithDefault(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxAssignmentTargetPropertyIdentifier(pos, ast) {
  return new AssignmentTargetPropertyIdentifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxAssignmentTargetPropertyProperty(pos, ast) {
  return new AssignmentTargetPropertyProperty(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionExpression(pos, ast) {
  if (ast.buffer[pos] === 51) return null;
  return constructExpression(pos, ast);
}

function constructBoxBlockStatement(pos, ast) {
  return new BlockStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxBreakStatement(pos, ast) {
  return new BreakStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxContinueStatement(pos, ast) {
  return new ContinueStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxDebuggerStatement(pos, ast) {
  return new DebuggerStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxDoWhileStatement(pos, ast) {
  return new DoWhileStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxEmptyStatement(pos, ast) {
  return new EmptyStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxExpressionStatement(pos, ast) {
  return new ExpressionStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxForInStatement(pos, ast) {
  return new ForInStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxForOfStatement(pos, ast) {
  return new ForOfStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxForStatement(pos, ast) {
  return new ForStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxIfStatement(pos, ast) {
  return new IfStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxLabeledStatement(pos, ast) {
  return new LabeledStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxReturnStatement(pos, ast) {
  return new ReturnStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxSwitchStatement(pos, ast) {
  return new SwitchStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxThrowStatement(pos, ast) {
  return new ThrowStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTryStatement(pos, ast) {
  return new TryStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxWhileStatement(pos, ast) {
  return new WhileStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxWithStatement(pos, ast) {
  return new WithStatement(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxVariableDeclaration(pos, ast) {
  return new VariableDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeAliasDeclaration(pos, ast) {
  return new TSTypeAliasDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSInterfaceDeclaration(pos, ast) {
  return new TSInterfaceDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSEnumDeclaration(pos, ast) {
  return new TSEnumDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSModuleDeclaration(pos, ast) {
  return new TSModuleDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSImportEqualsDeclaration(pos, ast) {
  return new TSImportEqualsDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecVariableDeclarator(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new VariableDeclarator(pos, ast));
    pos += 64;
  }
  return arr;
}

function constructOptionStatement(pos, ast) {
  if (ast.buffer[pos] === 70) return null;
  return constructStatement(pos, ast);
}

function constructOptionForStatementInit(pos, ast) {
  if (ast.buffer[pos] === 65) return null;
  return constructForStatementInit(pos, ast);
}

function constructOptionLabelIdentifier(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0) return null;
  return new LabelIdentifier(pos, ast);
}

function constructVecSwitchCase(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new SwitchCase(pos, ast));
    pos += 48;
  }
  return arr;
}

function constructBoxCatchClause(pos, ast) {
  return new CatchClause(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxCatchClause(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxCatchClause(pos, ast);
}

function constructOptionBoxBlockStatement(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxBlockStatement(pos, ast);
}

function constructOptionCatchParameter(pos, ast) {
  if (ast.buffer[pos + 32] === 2) return null;
  return new CatchParameter(pos, ast);
}

function constructBoxTSTypeAnnotation(pos, ast) {
  return new TSTypeAnnotation(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxTSTypeAnnotation(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxTSTypeAnnotation(pos, ast);
}

function constructBoxBindingIdentifier(pos, ast) {
  return new BindingIdentifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxObjectPattern(pos, ast) {
  return new ObjectPattern(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxArrayPattern(pos, ast) {
  return new ArrayPattern(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxAssignmentPattern(pos, ast) {
  return new AssignmentPattern(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecBindingProperty(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new BindingProperty(pos, ast));
    pos += 64;
  }
  return arr;
}

function constructBoxBindingRestElement(pos, ast) {
  return new BindingRestElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxBindingRestElement(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxBindingRestElement(pos, ast);
}

function constructOptionBindingPattern(pos, ast) {
  if (ast.buffer[pos + 24] === 2) return null;
  return new BindingPattern(pos, ast);
}

function constructVecOptionBindingPattern(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructOptionBindingPattern(pos, ast));
    pos += 32;
  }
  return arr;
}

function constructOptionBindingIdentifier(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0) return null;
  return new BindingIdentifier(pos, ast);
}

function constructBoxTSTypeParameterDeclaration(pos, ast) {
  return new TSTypeParameterDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxTSTypeParameterDeclaration(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxTSTypeParameterDeclaration(pos, ast);
}

function constructBoxTSThisParameter(pos, ast) {
  return new TSThisParameter(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxTSThisParameter(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxTSThisParameter(pos, ast);
}

function constructBoxFormalParameters(pos, ast) {
  return new FormalParameters(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxFunctionBody(pos, ast) {
  return new FunctionBody(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxFunctionBody(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxFunctionBody(pos, ast);
}

function constructVecFormalParameter(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new FormalParameter(pos, ast));
    pos += 72;
  }
  return arr;
}

function constructVecDecorator(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new Decorator(pos, ast));
    pos += 24;
  }
  return arr;
}

function constructOptionTSAccessibility(pos, ast) {
  if (ast.buffer[pos] === 3) return null;
  return constructTSAccessibility(pos, ast);
}

function constructVecTSClassImplements(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new TSClassImplements(pos, ast));
    pos += 32;
  }
  return arr;
}

function constructBoxClassBody(pos, ast) {
  return new ClassBody(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecClassElement(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructClassElement(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxStaticBlock(pos, ast) {
  return new StaticBlock(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxMethodDefinition(pos, ast) {
  return new MethodDefinition(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxPropertyDefinition(pos, ast) {
  return new PropertyDefinition(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxAccessorProperty(pos, ast) {
  return new AccessorProperty(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSIndexSignature(pos, ast) {
  return new TSIndexSignature(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxImportDeclaration(pos, ast) {
  return new ImportDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxExportAllDeclaration(pos, ast) {
  return new ExportAllDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxExportDefaultDeclaration(pos, ast) {
  return new ExportDefaultDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxExportNamedDeclaration(pos, ast) {
  return new ExportNamedDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSExportAssignment(pos, ast) {
  return new TSExportAssignment(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSNamespaceExportDeclaration(pos, ast) {
  return new TSNamespaceExportDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionImportPhase(pos, ast) {
  if (ast.buffer[pos] === 2) return null;
  return constructImportPhase(pos, ast);
}

function constructVecImportDeclarationSpecifier(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructImportDeclarationSpecifier(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructOptionVecImportDeclarationSpecifier(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructVecImportDeclarationSpecifier(pos, ast);
}

function constructBoxWithClause(pos, ast) {
  return new WithClause(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxWithClause(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxWithClause(pos, ast);
}

function constructBoxImportSpecifier(pos, ast) {
  return new ImportSpecifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxImportDefaultSpecifier(pos, ast) {
  return new ImportDefaultSpecifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxImportNamespaceSpecifier(pos, ast) {
  return new ImportNamespaceSpecifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecImportAttribute(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new ImportAttribute(pos, ast));
    pos += 112;
  }
  return arr;
}

function constructOptionDeclaration(pos, ast) {
  if (ast.buffer[pos] === 31) return null;
  return constructDeclaration(pos, ast);
}

function constructVecExportSpecifier(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new ExportSpecifier(pos, ast));
    pos += 128;
  }
  return arr;
}

function constructOptionStringLiteral(pos, ast) {
  if (ast.buffer[pos + 40] === 2) return null;
  return new StringLiteral(pos, ast);
}

function constructOptionModuleExportName(pos, ast) {
  if (ast.buffer[pos] === 3) return null;
  return constructModuleExportName(pos, ast);
}

function constructF64(pos, ast) {
  return ast.buffer.float64[pos >> 3];
}

function constructU8(pos, ast) {
  return ast.buffer[pos];
}

function constructBoxJSXOpeningElement(pos, ast) {
  return new JSXOpeningElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecJSXChild(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructJSXChild(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxJSXClosingElement(pos, ast) {
  return new JSXClosingElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxJSXClosingElement(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxJSXClosingElement(pos, ast);
}

function constructVecJSXAttributeItem(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructJSXAttributeItem(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxJSXIdentifier(pos, ast) {
  return new JSXIdentifier(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXNamespacedName(pos, ast) {
  return new JSXNamespacedName(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXMemberExpression(pos, ast) {
  return new JSXMemberExpression(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXAttribute(pos, ast) {
  return new JSXAttribute(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXSpreadAttribute(pos, ast) {
  return new JSXSpreadAttribute(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionJSXAttributeValue(pos, ast) {
  if (ast.buffer[pos] === 4) return null;
  return constructJSXAttributeValue(pos, ast);
}

function constructBoxJSXExpressionContainer(pos, ast) {
  return new JSXExpressionContainer(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXText(pos, ast) {
  return new JSXText(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSXSpreadChild(pos, ast) {
  return new JSXSpreadChild(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecTSEnumMember(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new TSEnumMember(pos, ast));
    pos += 40;
  }
  return arr;
}

function constructBoxTSAnyKeyword(pos, ast) {
  return new TSAnyKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSBigIntKeyword(pos, ast) {
  return new TSBigIntKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSBooleanKeyword(pos, ast) {
  return new TSBooleanKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSIntrinsicKeyword(pos, ast) {
  return new TSIntrinsicKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSNeverKeyword(pos, ast) {
  return new TSNeverKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSNullKeyword(pos, ast) {
  return new TSNullKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSNumberKeyword(pos, ast) {
  return new TSNumberKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSObjectKeyword(pos, ast) {
  return new TSObjectKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSStringKeyword(pos, ast) {
  return new TSStringKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSSymbolKeyword(pos, ast) {
  return new TSSymbolKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSUndefinedKeyword(pos, ast) {
  return new TSUndefinedKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSUnknownKeyword(pos, ast) {
  return new TSUnknownKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSVoidKeyword(pos, ast) {
  return new TSVoidKeyword(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSArrayType(pos, ast) {
  return new TSArrayType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSConditionalType(pos, ast) {
  return new TSConditionalType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSConstructorType(pos, ast) {
  return new TSConstructorType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSFunctionType(pos, ast) {
  return new TSFunctionType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSImportType(pos, ast) {
  return new TSImportType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSIndexedAccessType(pos, ast) {
  return new TSIndexedAccessType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSInferType(pos, ast) {
  return new TSInferType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSIntersectionType(pos, ast) {
  return new TSIntersectionType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSLiteralType(pos, ast) {
  return new TSLiteralType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSMappedType(pos, ast) {
  return new TSMappedType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSNamedTupleMember(pos, ast) {
  return new TSNamedTupleMember(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTemplateLiteralType(pos, ast) {
  return new TSTemplateLiteralType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSThisType(pos, ast) {
  return new TSThisType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTupleType(pos, ast) {
  return new TSTupleType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeLiteral(pos, ast) {
  return new TSTypeLiteral(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeOperator(pos, ast) {
  return new TSTypeOperator(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypePredicate(pos, ast) {
  return new TSTypePredicate(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeQuery(pos, ast) {
  return new TSTypeQuery(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeReference(pos, ast) {
  return new TSTypeReference(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSUnionType(pos, ast) {
  return new TSUnionType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSParenthesizedType(pos, ast) {
  return new TSParenthesizedType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSDocNullableType(pos, ast) {
  return new JSDocNullableType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSDocNonNullableType(pos, ast) {
  return new JSDocNonNullableType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxJSDocUnknownType(pos, ast) {
  return new JSDocUnknownType(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecTSType(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructTSType(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructVecTSTupleElement(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructTSTupleElement(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxTSOptionalType(pos, ast) {
  return new TSOptionalType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSRestType(pos, ast) {
  return new TSRestType(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSQualifiedName(pos, ast) {
  return new TSQualifiedName(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionTSType(pos, ast) {
  if (ast.buffer[pos] === 38) return null;
  return constructTSType(pos, ast);
}

function constructVecTSTypeParameter(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new TSTypeParameter(pos, ast));
    pos += 80;
  }
  return arr;
}

function constructVecTSInterfaceHeritage(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new TSInterfaceHeritage(pos, ast));
    pos += 32;
  }
  return arr;
}

function constructBoxTSInterfaceBody(pos, ast) {
  return new TSInterfaceBody(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecTSSignature(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(constructTSSignature(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructBoxTSPropertySignature(pos, ast) {
  return new TSPropertySignature(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSCallSignatureDeclaration(pos, ast) {
  return new TSCallSignatureDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSConstructSignatureDeclaration(pos, ast) {
  return new TSConstructSignatureDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSMethodSignature(pos, ast) {
  return new TSMethodSignature(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecTSIndexSignatureName(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new TSIndexSignatureName(pos, ast));
    pos += 32;
  }
  return arr;
}

function constructOptionTSModuleDeclarationBody(pos, ast) {
  if (ast.buffer[pos] === 2) return null;
  return constructTSModuleDeclarationBody(pos, ast);
}

function constructBoxTSModuleBlock(pos, ast) {
  return new TSModuleBlock(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSTypeParameter(pos, ast) {
  return new TSTypeParameter(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxObjectExpression(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxObjectExpression(pos, ast);
}

function constructOptionTSTypeName(pos, ast) {
  if (ast.buffer[pos] === 2) return null;
  return constructTSTypeName(pos, ast);
}

function constructOptionTSMappedTypeModifierOperator(pos, ast) {
  if (ast.buffer[pos] === 3) return null;
  return constructTSMappedTypeModifierOperator(pos, ast);
}

function constructBoxTSExternalModuleReference(pos, ast) {
  return new TSExternalModuleReference(ast.buffer.uint32[pos >> 2], ast);
}

function constructU32(pos, ast) {
  return ast.buffer.uint32[pos >> 2];
}

function constructOptionNameSpan(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0) return null;
  return new NameSpan(pos, ast);
}

function constructU64(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return uint32[pos32] + uint32[pos32 + 1] * 4294967296;
}

function constructOptionU64(pos, ast) {
  if (ast.buffer[pos] === 0) return null;
  return constructU64(pos + 8, ast);
}

function constructVecError(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new Error(pos, ast));
    pos += 80;
  }
  return arr;
}

function constructVecErrorLabel(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new ErrorLabel(pos, ast));
    pos += 24;
  }
  return arr;
}

function constructVecStaticImport(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new StaticImport(pos, ast));
    pos += 56;
  }
  return arr;
}

function constructVecStaticExport(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new StaticExport(pos, ast));
    pos += 32;
  }
  return arr;
}

function constructVecDynamicImport(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new DynamicImport(pos, ast));
    pos += 16;
  }
  return arr;
}

function constructVecSpan(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new Span(pos, ast));
    pos += 8;
  }
  return arr;
}

function constructVecImportEntry(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new ImportEntry(pos, ast));
    pos += 96;
  }
  return arr;
}

function constructVecExportEntry(pos, ast) {
  const { uint32 } = ast.buffer,
    arr = [],
    pos32 = pos >> 2,
    len = uint32[pos32 + 2];
  pos = uint32[pos32];
  for (let i = 0; i < len; i++) {
    arr.push(new ExportEntry(pos, ast));
    pos += 144;
  }
  return arr;
}
