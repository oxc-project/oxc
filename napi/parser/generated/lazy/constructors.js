// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer_lazy.rs`.

import { constructorError, TOKEN } from "../../src-js/raw-transfer/lazy-common.js";
import { NodeArray } from "../../src-js/raw-transfer/node-array.js";

const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true }),
  decodeStr = textDecoder.decode.bind(textDecoder),
  { fromCodePoint } = String,
  inspectSymbol = Symbol.for("nodejs.util.inspect.custom");

export class Program {
  type = "Program";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get sourceType() {
    const internal = this.#internal;
    return new SourceType(internal.pos + 124, internal.ast);
  }

  get hashbang() {
    const internal = this.#internal;
    return constructOptionHashbang(internal.pos + 48, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecStatement(internal.pos + 96, internal.ast));
  }

  toJSON() {
    return {
      type: "Program",
      start: this.start,
      end: this.end,
      sourceType: this.sourceType,
      hashbang: this.hashbang,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugProgram.prototype);
  }
}

const DebugProgram = class Program {};

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

export class IdentifierName {
  type = "IdentifierName";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "IdentifierName",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugIdentifierName.prototype);
  }
}

const DebugIdentifierName = class IdentifierName {};

export class IdentifierReference {
  type = "IdentifierReference";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "IdentifierReference",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugIdentifierReference.prototype);
  }
}

const DebugIdentifierReference = class IdentifierReference {};

export class BindingIdentifier {
  type = "BindingIdentifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "BindingIdentifier",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBindingIdentifier.prototype);
  }
}

const DebugBindingIdentifier = class BindingIdentifier {};

export class LabelIdentifier {
  type = "LabelIdentifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "LabelIdentifier",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugLabelIdentifier.prototype);
  }
}

const DebugLabelIdentifier = class LabelIdentifier {};

export class ThisExpression {
  type = "ThisExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "ThisExpression",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugThisExpression.prototype);
  }
}

const DebugThisExpression = class ThisExpression {};

export class ArrayExpression {
  type = "ArrayExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $elements: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get elements() {
    const internal = this.#internal,
      cached = internal.$elements;
    if (cached !== void 0) return cached;
    return (internal.$elements = constructVecArrayExpressionElement(
      internal.pos + 8,
      internal.ast,
    ));
  }

  toJSON() {
    return {
      type: "ArrayExpression",
      start: this.start,
      end: this.end,
      elements: this.elements,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugArrayExpression.prototype);
  }
}

const DebugArrayExpression = class ArrayExpression {};

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

export class Elision {
  type = "Elision";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "Elision",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugElision.prototype);
  }
}

const DebugElision = class Elision {};

export class ObjectExpression {
  type = "ObjectExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $properties: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get properties() {
    const internal = this.#internal,
      cached = internal.$properties;
    if (cached !== void 0) return cached;
    return (internal.$properties = constructVecObjectPropertyKind(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "ObjectExpression",
      start: this.start,
      end: this.end,
      properties: this.properties,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugObjectExpression.prototype);
  }
}

const DebugObjectExpression = class ObjectExpression {};

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

export class ObjectProperty {
  type = "ObjectProperty";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructPropertyKind(internal.pos + 40, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 8, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  get method() {
    const internal = this.#internal;
    return constructBool(internal.pos + 41, internal.ast);
  }

  get shorthand() {
    const internal = this.#internal;
    return constructBool(internal.pos + 42, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 43, internal.ast);
  }

  toJSON() {
    return {
      type: "ObjectProperty",
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugObjectProperty.prototype);
  }
}

const DebugObjectProperty = class ObjectProperty {};

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
      return "init";
    case 1:
      return "get";
    case 2:
      return "set";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyKind`);
  }
}

export class TemplateLiteral {
  type = "TemplateLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $quasis: void 0, $expressions: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get quasis() {
    const internal = this.#internal,
      cached = internal.$quasis;
    if (cached !== void 0) return cached;
    return (internal.$quasis = constructVecTemplateElement(internal.pos + 8, internal.ast));
  }

  get expressions() {
    const internal = this.#internal,
      cached = internal.$expressions;
    if (cached !== void 0) return cached;
    return (internal.$expressions = constructVecExpression(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "TemplateLiteral",
      start: this.start,
      end: this.end,
      quasis: this.quasis,
      expressions: this.expressions,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTemplateLiteral.prototype);
  }
}

const DebugTemplateLiteral = class TemplateLiteral {};

export class TaggedTemplateExpression {
  type = "TaggedTemplateExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get tag() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  get quasi() {
    const internal = this.#internal;
    return new TemplateLiteral(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "TaggedTemplateExpression",
      start: this.start,
      end: this.end,
      tag: this.tag,
      typeArguments: this.typeArguments,
      quasi: this.quasi,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTaggedTemplateExpression.prototype);
  }
}

const DebugTaggedTemplateExpression = class TaggedTemplateExpression {};

export class TemplateElement {
  type = "TemplateElement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return new TemplateElementValue(internal.pos + 8, internal.ast);
  }

  get tail() {
    const internal = this.#internal;
    return constructBool(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "TemplateElement",
      start: this.start,
      end: this.end,
      value: this.value,
      tail: this.tail,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTemplateElement.prototype);
  }
}

const DebugTemplateElement = class TemplateElement {};

export class TemplateElementValue {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $raw: void 0, $cooked: void 0 };
    nodes.set(pos, this);
  }

  get raw() {
    const internal = this.#internal,
      cached = internal.$raw;
    if (cached !== void 0) return cached;
    return (internal.$raw = constructStr(internal.pos, internal.ast));
  }

  get cooked() {
    const internal = this.#internal,
      cached = internal.$cooked;
    if (cached !== void 0) return cached;
    return (internal.$cooked = constructOptionStr(internal.pos + 16, internal.ast));
  }

  toJSON() {
    return {
      raw: this.raw,
      cooked: this.cooked,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTemplateElementValue.prototype);
  }
}

const DebugTemplateElementValue = class TemplateElementValue {};

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

export class ComputedMemberExpression {
  type = "ComputedMemberExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get property() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "ComputedMemberExpression",
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
      optional: this.optional,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugComputedMemberExpression.prototype);
  }
}

const DebugComputedMemberExpression = class ComputedMemberExpression {};

export class StaticMemberExpression {
  type = "StaticMemberExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get property() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 24, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 48, internal.ast);
  }

  toJSON() {
    return {
      type: "StaticMemberExpression",
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
      optional: this.optional,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugStaticMemberExpression.prototype);
  }
}

const DebugStaticMemberExpression = class StaticMemberExpression {};

export class PrivateFieldExpression {
  type = "PrivateFieldExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get property() {
    const internal = this.#internal;
    return new PrivateIdentifier(internal.pos + 24, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 48, internal.ast);
  }

  toJSON() {
    return {
      type: "PrivateFieldExpression",
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
      optional: this.optional,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugPrivateFieldExpression.prototype);
  }
}

const DebugPrivateFieldExpression = class PrivateFieldExpression {};

export class CallExpression {
  type = "CallExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $arguments: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get callee() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  get arguments() {
    const internal = this.#internal,
      cached = internal.$arguments;
    if (cached !== void 0) return cached;
    return (internal.$arguments = constructVecArgument(internal.pos + 32, internal.ast));
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 56, internal.ast);
  }

  toJSON() {
    return {
      type: "CallExpression",
      start: this.start,
      end: this.end,
      callee: this.callee,
      typeArguments: this.typeArguments,
      arguments: this.arguments,
      optional: this.optional,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugCallExpression.prototype);
  }
}

const DebugCallExpression = class CallExpression {};

export class NewExpression {
  type = "NewExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $arguments: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get callee() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  get arguments() {
    const internal = this.#internal,
      cached = internal.$arguments;
    if (cached !== void 0) return cached;
    return (internal.$arguments = constructVecArgument(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "NewExpression",
      start: this.start,
      end: this.end,
      callee: this.callee,
      typeArguments: this.typeArguments,
      arguments: this.arguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugNewExpression.prototype);
  }
}

const DebugNewExpression = class NewExpression {};

export class MetaProperty {
  type = "MetaProperty";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get meta() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 8, internal.ast);
  }

  get property() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "MetaProperty",
      start: this.start,
      end: this.end,
      meta: this.meta,
      property: this.property,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugMetaProperty.prototype);
  }
}

const DebugMetaProperty = class MetaProperty {};

export class SpreadElement {
  type = "SpreadElement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "SpreadElement",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSpreadElement.prototype);
  }
}

const DebugSpreadElement = class SpreadElement {};

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

export class UpdateExpression {
  type = "UpdateExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructUpdateOperator(internal.pos + 24, internal.ast);
  }

  get prefix() {
    const internal = this.#internal;
    return constructBool(internal.pos + 25, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructSimpleAssignmentTarget(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "UpdateExpression",
      start: this.start,
      end: this.end,
      operator: this.operator,
      prefix: this.prefix,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugUpdateExpression.prototype);
  }
}

const DebugUpdateExpression = class UpdateExpression {};

export class UnaryExpression {
  type = "UnaryExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructUnaryOperator(internal.pos + 24, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "UnaryExpression",
      start: this.start,
      end: this.end,
      operator: this.operator,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugUnaryExpression.prototype);
  }
}

const DebugUnaryExpression = class UnaryExpression {};

export class BinaryExpression {
  type = "BinaryExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructBinaryOperator(internal.pos + 40, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "BinaryExpression",
      start: this.start,
      end: this.end,
      left: this.left,
      operator: this.operator,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBinaryExpression.prototype);
  }
}

const DebugBinaryExpression = class BinaryExpression {};

export class PrivateInExpression {
  type = "PrivateInExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return new PrivateIdentifier(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "PrivateInExpression",
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugPrivateInExpression.prototype);
  }
}

const DebugPrivateInExpression = class PrivateInExpression {};

export class LogicalExpression {
  type = "LogicalExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructLogicalOperator(internal.pos + 40, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "LogicalExpression",
      start: this.start,
      end: this.end,
      left: this.left,
      operator: this.operator,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugLogicalExpression.prototype);
  }
}

const DebugLogicalExpression = class LogicalExpression {};

export class ConditionalExpression {
  type = "ConditionalExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get consequent() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  get alternate() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "ConditionalExpression",
      start: this.start,
      end: this.end,
      test: this.test,
      consequent: this.consequent,
      alternate: this.alternate,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugConditionalExpression.prototype);
  }
}

const DebugConditionalExpression = class ConditionalExpression {};

export class AssignmentExpression {
  type = "AssignmentExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructAssignmentOperator(internal.pos + 40, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructAssignmentTarget(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "AssignmentExpression",
      start: this.start,
      end: this.end,
      operator: this.operator,
      left: this.left,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAssignmentExpression.prototype);
  }
}

const DebugAssignmentExpression = class AssignmentExpression {};

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

export class ArrayAssignmentTarget {
  type = "ArrayAssignmentTarget";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $elements: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get elements() {
    const internal = this.#internal,
      cached = internal.$elements;
    if (cached !== void 0) return cached;
    return (internal.$elements = constructVecOptionAssignmentTargetMaybeDefault(
      internal.pos + 8,
      internal.ast,
    ));
  }

  toJSON() {
    return {
      type: "ArrayAssignmentTarget",
      start: this.start,
      end: this.end,
      elements: this.elements,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugArrayAssignmentTarget.prototype);
  }
}

const DebugArrayAssignmentTarget = class ArrayAssignmentTarget {};

export class ObjectAssignmentTarget {
  type = "ObjectAssignmentTarget";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $properties: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get properties() {
    const internal = this.#internal,
      cached = internal.$properties;
    if (cached !== void 0) return cached;
    return (internal.$properties = constructVecAssignmentTargetProperty(
      internal.pos + 8,
      internal.ast,
    ));
  }

  toJSON() {
    return {
      type: "ObjectAssignmentTarget",
      start: this.start,
      end: this.end,
      properties: this.properties,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugObjectAssignmentTarget.prototype);
  }
}

const DebugObjectAssignmentTarget = class ObjectAssignmentTarget {};

export class AssignmentTargetRest {
  type = "AssignmentTargetRest";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructAssignmentTarget(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "AssignmentTargetRest",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAssignmentTargetRest.prototype);
  }
}

const DebugAssignmentTargetRest = class AssignmentTargetRest {};

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
      throw new Error(
        `Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetMaybeDefault`,
      );
  }
}

export class AssignmentTargetWithDefault {
  type = "AssignmentTargetWithDefault";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructAssignmentTarget(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "AssignmentTargetWithDefault",
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAssignmentTargetWithDefault.prototype);
  }
}

const DebugAssignmentTargetWithDefault = class AssignmentTargetWithDefault {};

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

export class AssignmentTargetPropertyIdentifier {
  type = "AssignmentTargetPropertyIdentifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return new IdentifierReference(internal.pos + 8, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "AssignmentTargetPropertyIdentifier",
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAssignmentTargetPropertyIdentifier.prototype);
  }
}

const DebugAssignmentTargetPropertyIdentifier = class AssignmentTargetPropertyIdentifier {};

export class AssignmentTargetPropertyProperty {
  type = "AssignmentTargetPropertyProperty";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 8, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructAssignmentTargetMaybeDefault(internal.pos + 24, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "AssignmentTargetPropertyProperty",
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
      computed: this.computed,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAssignmentTargetPropertyProperty.prototype);
  }
}

const DebugAssignmentTargetPropertyProperty = class AssignmentTargetPropertyProperty {};

export class SequenceExpression {
  type = "SequenceExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $expressions: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expressions() {
    const internal = this.#internal,
      cached = internal.$expressions;
    if (cached !== void 0) return cached;
    return (internal.$expressions = constructVecExpression(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "SequenceExpression",
      start: this.start,
      end: this.end,
      expressions: this.expressions,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSequenceExpression.prototype);
  }
}

const DebugSequenceExpression = class SequenceExpression {};

export class Super {
  type = "Super";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "Super",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSuper.prototype);
  }
}

const DebugSuper = class Super {};

export class AwaitExpression {
  type = "AwaitExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "AwaitExpression",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAwaitExpression.prototype);
  }
}

const DebugAwaitExpression = class AwaitExpression {};

export class ChainExpression {
  type = "ChainExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructChainElement(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ChainExpression",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugChainExpression.prototype);
  }
}

const DebugChainExpression = class ChainExpression {};

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

export class ParenthesizedExpression {
  type = "ParenthesizedExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ParenthesizedExpression",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugParenthesizedExpression.prototype);
  }
}

const DebugParenthesizedExpression = class ParenthesizedExpression {};

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
      return constructBoxTSGlobalDeclaration(pos + 8, ast);
    case 40:
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

export class Directive {
  type = "Directive";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $directive: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return new StringLiteral(internal.pos + 8, internal.ast);
  }

  get directive() {
    const internal = this.#internal,
      cached = internal.$directive;
    if (cached !== void 0) return cached;
    return (internal.$directive = constructStr(internal.pos + 56, internal.ast));
  }

  toJSON() {
    return {
      type: "Directive",
      start: this.start,
      end: this.end,
      expression: this.expression,
      directive: this.directive,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugDirective.prototype);
  }
}

const DebugDirective = class Directive {};

export class Hashbang {
  type = "Hashbang";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $value: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal,
      cached = internal.$value;
    if (cached !== void 0) return cached;
    return (internal.$value = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "Hashbang",
      start: this.start,
      end: this.end,
      value: this.value,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugHashbang.prototype);
  }
}

const DebugHashbang = class Hashbang {};

export class BlockStatement {
  type = "BlockStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecStatement(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "BlockStatement",
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBlockStatement.prototype);
  }
}

const DebugBlockStatement = class BlockStatement {};

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
      return constructBoxTSGlobalDeclaration(pos + 8, ast);
    case 40:
      return constructBoxTSImportEqualsDeclaration(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Declaration`);
  }
}

export class VariableDeclaration {
  type = "VariableDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $declarations: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructVariableDeclarationKind(internal.pos + 32, internal.ast);
  }

  get declarations() {
    const internal = this.#internal,
      cached = internal.$declarations;
    if (cached !== void 0) return cached;
    return (internal.$declarations = constructVecVariableDeclarator(
      internal.pos + 8,
      internal.ast,
    ));
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 33, internal.ast);
  }

  toJSON() {
    return {
      type: "VariableDeclaration",
      start: this.start,
      end: this.end,
      kind: this.kind,
      declarations: this.declarations,
      declare: this.declare,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugVariableDeclaration.prototype);
  }
}

const DebugVariableDeclaration = class VariableDeclaration {};

function constructVariableDeclarationKind(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for VariableDeclarationKind`);
  }
}

export class VariableDeclarator {
  type = "VariableDeclarator";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingPattern(internal.pos + 8, internal.ast);
  }

  get init() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 40, internal.ast);
  }

  get definite() {
    const internal = this.#internal;
    return constructBool(internal.pos + 57, internal.ast);
  }

  toJSON() {
    return {
      type: "VariableDeclarator",
      start: this.start,
      end: this.end,
      id: this.id,
      init: this.init,
      definite: this.definite,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugVariableDeclarator.prototype);
  }
}

const DebugVariableDeclarator = class VariableDeclarator {};

export class EmptyStatement {
  type = "EmptyStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "EmptyStatement",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugEmptyStatement.prototype);
  }
}

const DebugEmptyStatement = class EmptyStatement {};

export class ExpressionStatement {
  type = "ExpressionStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ExpressionStatement",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugExpressionStatement.prototype);
  }
}

const DebugExpressionStatement = class ExpressionStatement {};

export class IfStatement {
  type = "IfStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get consequent() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 24, internal.ast);
  }

  get alternate() {
    const internal = this.#internal;
    return constructOptionStatement(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "IfStatement",
      start: this.start,
      end: this.end,
      test: this.test,
      consequent: this.consequent,
      alternate: this.alternate,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugIfStatement.prototype);
  }
}

const DebugIfStatement = class IfStatement {};

export class DoWhileStatement {
  type = "DoWhileStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 8, internal.ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "DoWhileStatement",
      start: this.start,
      end: this.end,
      body: this.body,
      test: this.test,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugDoWhileStatement.prototype);
  }
}

const DebugDoWhileStatement = class DoWhileStatement {};

export class WhileStatement {
  type = "WhileStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get test() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "WhileStatement",
      start: this.start,
      end: this.end,
      test: this.test,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugWhileStatement.prototype);
  }
}

const DebugWhileStatement = class WhileStatement {};

export class ForStatement {
  type = "ForStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get init() {
    const internal = this.#internal;
    return constructOptionForStatementInit(internal.pos + 8, internal.ast);
  }

  get test() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 24, internal.ast);
  }

  get update() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 40, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 56, internal.ast);
  }

  toJSON() {
    return {
      type: "ForStatement",
      start: this.start,
      end: this.end,
      init: this.init,
      test: this.test,
      update: this.update,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugForStatement.prototype);
  }
}

const DebugForStatement = class ForStatement {};

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

export class ForInStatement {
  type = "ForInStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructForStatementLeft(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "ForInStatement",
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugForInStatement.prototype);
  }
}

const DebugForInStatement = class ForInStatement {};

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

export class ForOfStatement {
  type = "ForOfStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get await() {
    const internal = this.#internal;
    return constructBool(internal.pos + 60, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructForStatementLeft(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "ForOfStatement",
      start: this.start,
      end: this.end,
      await: this.await,
      left: this.left,
      right: this.right,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugForOfStatement.prototype);
  }
}

const DebugForOfStatement = class ForOfStatement {};

export class ContinueStatement {
  type = "ContinueStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get label() {
    const internal = this.#internal;
    return constructOptionLabelIdentifier(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ContinueStatement",
      start: this.start,
      end: this.end,
      label: this.label,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugContinueStatement.prototype);
  }
}

const DebugContinueStatement = class ContinueStatement {};

export class BreakStatement {
  type = "BreakStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get label() {
    const internal = this.#internal;
    return constructOptionLabelIdentifier(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "BreakStatement",
      start: this.start,
      end: this.end,
      label: this.label,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBreakStatement.prototype);
  }
}

const DebugBreakStatement = class BreakStatement {};

export class ReturnStatement {
  type = "ReturnStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ReturnStatement",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugReturnStatement.prototype);
  }
}

const DebugReturnStatement = class ReturnStatement {};

export class WithStatement {
  type = "WithStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get object() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "WithStatement",
      start: this.start,
      end: this.end,
      object: this.object,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugWithStatement.prototype);
  }
}

const DebugWithStatement = class WithStatement {};

export class SwitchStatement {
  type = "SwitchStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $cases: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get discriminant() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get cases() {
    const internal = this.#internal,
      cached = internal.$cases;
    if (cached !== void 0) return cached;
    return (internal.$cases = constructVecSwitchCase(internal.pos + 24, internal.ast));
  }

  toJSON() {
    return {
      type: "SwitchStatement",
      start: this.start,
      end: this.end,
      discriminant: this.discriminant,
      cases: this.cases,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSwitchStatement.prototype);
  }
}

const DebugSwitchStatement = class SwitchStatement {};

export class SwitchCase {
  type = "SwitchCase";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $consequent: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get test() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 8, internal.ast);
  }

  get consequent() {
    const internal = this.#internal,
      cached = internal.$consequent;
    if (cached !== void 0) return cached;
    return (internal.$consequent = constructVecStatement(internal.pos + 24, internal.ast));
  }

  toJSON() {
    return {
      type: "SwitchCase",
      start: this.start,
      end: this.end,
      test: this.test,
      consequent: this.consequent,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSwitchCase.prototype);
  }
}

const DebugSwitchCase = class SwitchCase {};

export class LabeledStatement {
  type = "LabeledStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get label() {
    const internal = this.#internal;
    return new LabelIdentifier(internal.pos + 8, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructStatement(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "LabeledStatement",
      start: this.start,
      end: this.end,
      label: this.label,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugLabeledStatement.prototype);
  }
}

const DebugLabeledStatement = class LabeledStatement {};

export class ThrowStatement {
  type = "ThrowStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ThrowStatement",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugThrowStatement.prototype);
  }
}

const DebugThrowStatement = class ThrowStatement {};

export class TryStatement {
  type = "TryStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get block() {
    const internal = this.#internal;
    return constructBoxBlockStatement(internal.pos + 8, internal.ast);
  }

  get handler() {
    const internal = this.#internal;
    return constructOptionBoxCatchClause(internal.pos + 16, internal.ast);
  }

  get finalizer() {
    const internal = this.#internal;
    return constructOptionBoxBlockStatement(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TryStatement",
      start: this.start,
      end: this.end,
      block: this.block,
      handler: this.handler,
      finalizer: this.finalizer,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTryStatement.prototype);
  }
}

const DebugTryStatement = class TryStatement {};

export class CatchClause {
  type = "CatchClause";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get param() {
    const internal = this.#internal;
    return constructOptionCatchParameter(internal.pos + 8, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructBoxBlockStatement(internal.pos + 48, internal.ast);
  }

  toJSON() {
    return {
      type: "CatchClause",
      start: this.start,
      end: this.end,
      param: this.param,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugCatchClause.prototype);
  }
}

const DebugCatchClause = class CatchClause {};

export class CatchParameter {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get pattern() {
    const internal = this.#internal;
    return new BindingPattern(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      pattern: this.pattern,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugCatchParameter.prototype);
  }
}

const DebugCatchParameter = class CatchParameter {};

export class DebuggerStatement {
  type = "DebuggerStatement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "DebuggerStatement",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugDebuggerStatement.prototype);
  }
}

const DebugDebuggerStatement = class DebuggerStatement {};

export class BindingPattern {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get kind() {
    const internal = this.#internal;
    return constructBindingPatternKind(internal.pos, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 16, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      kind: this.kind,
      typeAnnotation: this.typeAnnotation,
      optional: this.optional,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBindingPattern.prototype);
  }
}

const DebugBindingPattern = class BindingPattern {};

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

export class AssignmentPattern {
  type = "AssignmentPattern";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return new BindingPattern(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "AssignmentPattern",
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAssignmentPattern.prototype);
  }
}

const DebugAssignmentPattern = class AssignmentPattern {};

export class ObjectPattern {
  type = "ObjectPattern";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $properties: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get properties() {
    const internal = this.#internal,
      cached = internal.$properties;
    if (cached !== void 0) return cached;
    return (internal.$properties = constructVecBindingProperty(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "ObjectPattern",
      start: this.start,
      end: this.end,
      properties: this.properties,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugObjectPattern.prototype);
  }
}

const DebugObjectPattern = class ObjectPattern {};

export class BindingProperty {
  type = "BindingProperty";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 8, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return new BindingPattern(internal.pos + 24, internal.ast);
  }

  get shorthand() {
    const internal = this.#internal;
    return constructBool(internal.pos + 56, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 57, internal.ast);
  }

  toJSON() {
    return {
      type: "BindingProperty",
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
      shorthand: this.shorthand,
      computed: this.computed,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBindingProperty.prototype);
  }
}

const DebugBindingProperty = class BindingProperty {};

export class ArrayPattern {
  type = "ArrayPattern";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $elements: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get elements() {
    const internal = this.#internal,
      cached = internal.$elements;
    if (cached !== void 0) return cached;
    return (internal.$elements = constructVecOptionBindingPattern(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "ArrayPattern",
      start: this.start,
      end: this.end,
      elements: this.elements,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugArrayPattern.prototype);
  }
}

const DebugArrayPattern = class ArrayPattern {};

export class BindingRestElement {
  type = "BindingRestElement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return new BindingPattern(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "BindingRestElement",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBindingRestElement.prototype);
  }
}

const DebugBindingRestElement = class BindingRestElement {};

export class Function {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get type() {
    const internal = this.#internal;
    return constructFunctionType(internal.pos + 84, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return constructOptionBindingIdentifier(internal.pos + 8, internal.ast);
  }

  get generator() {
    const internal = this.#internal;
    return constructBool(internal.pos + 85, internal.ast);
  }

  get async() {
    const internal = this.#internal;
    return constructBool(internal.pos + 86, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 87, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 40, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 56, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 64, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructOptionBoxFunctionBody(internal.pos + 72, internal.ast);
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugFunction.prototype);
  }
}

const DebugFunction = class Function {};

function constructFunctionType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "FunctionDeclaration";
    case 1:
      return "FunctionExpression";
    case 2:
      return "TSDeclareFunction";
    case 3:
      return "TSEmptyBodyFunctionExpression";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for FunctionType`);
  }
}

export class FormalParameters {
  type = "FormalParameters";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $items: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructFormalParameterKind(internal.pos + 40, internal.ast);
  }

  get items() {
    const internal = this.#internal,
      cached = internal.$items;
    if (cached !== void 0) return cached;
    return (internal.$items = constructVecFormalParameter(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "FormalParameters",
      start: this.start,
      end: this.end,
      kind: this.kind,
      items: this.items,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugFormalParameters.prototype);
  }
}

const DebugFormalParameters = class FormalParameters {};

export class FormalParameter {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $decorators: void 0 };
    nodes.set(pos, this);
  }

  get decorators() {
    const internal = this.#internal,
      cached = internal.$decorators;
    if (cached !== void 0) return cached;
    return (internal.$decorators = constructVecDecorator(internal.pos + 8, internal.ast));
  }

  get pattern() {
    const internal = this.#internal;
    return new BindingPattern(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      decorators: this.decorators,
      pattern: this.pattern,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugFormalParameter.prototype);
  }
}

const DebugFormalParameter = class FormalParameter {};

function constructFormalParameterKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "FormalParameter";
    case 1:
      return "UniqueFormalParameters";
    case 2:
      return "ArrowFormalParameters";
    case 3:
      return "Signature";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for FormalParameterKind`);
  }
}

export class FunctionBody {
  type = "FunctionBody";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecStatement(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "FunctionBody",
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugFunctionBody.prototype);
  }
}

const DebugFunctionBody = class FunctionBody {};

export class ArrowFunctionExpression {
  type = "ArrowFunctionExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructBool(internal.pos + 44, internal.ast);
  }

  get async() {
    const internal = this.#internal;
    return constructBool(internal.pos + 45, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 8, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 16, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 24, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructBoxFunctionBody(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "ArrowFunctionExpression",
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugArrowFunctionExpression.prototype);
  }
}

const DebugArrowFunctionExpression = class ArrowFunctionExpression {};

export class YieldExpression {
  type = "YieldExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get delegate() {
    const internal = this.#internal;
    return constructBool(internal.pos + 24, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "YieldExpression",
      start: this.start,
      end: this.end,
      delegate: this.delegate,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugYieldExpression.prototype);
  }
}

const DebugYieldExpression = class YieldExpression {};

export class Class {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $decorators: void 0, $implements: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get type() {
    const internal = this.#internal;
    return constructClassType(internal.pos + 132, internal.ast);
  }

  get decorators() {
    const internal = this.#internal,
      cached = internal.$decorators;
    if (cached !== void 0) return cached;
    return (internal.$decorators = constructVecDecorator(internal.pos + 8, internal.ast));
  }

  get id() {
    const internal = this.#internal;
    return constructOptionBindingIdentifier(internal.pos + 32, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 64, internal.ast);
  }

  get superClass() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 72, internal.ast);
  }

  get superTypeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 88, internal.ast);
  }

  get implements() {
    const internal = this.#internal,
      cached = internal.$implements;
    if (cached !== void 0) return cached;
    return (internal.$implements = constructVecTSClassImplements(internal.pos + 96, internal.ast));
  }

  get body() {
    const internal = this.#internal;
    return constructBoxClassBody(internal.pos + 120, internal.ast);
  }

  get abstract() {
    const internal = this.#internal;
    return constructBool(internal.pos + 133, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 134, internal.ast);
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugClass.prototype);
  }
}

const DebugClass = class Class {};

function constructClassType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "ClassDeclaration";
    case 1:
      return "ClassExpression";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ClassType`);
  }
}

export class ClassBody {
  type = "ClassBody";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecClassElement(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "ClassBody",
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugClassBody.prototype);
  }
}

const DebugClassBody = class ClassBody {};

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

export class MethodDefinition {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $decorators: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get type() {
    const internal = this.#internal;
    return constructMethodDefinitionType(internal.pos + 56, internal.ast);
  }

  get decorators() {
    const internal = this.#internal,
      cached = internal.$decorators;
    if (cached !== void 0) return cached;
    return (internal.$decorators = constructVecDecorator(internal.pos + 8, internal.ast));
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 32, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructBoxFunction(internal.pos + 48, internal.ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructMethodDefinitionKind(internal.pos + 57, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 58, internal.ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.pos + 59, internal.ast);
  }

  get override() {
    const internal = this.#internal;
    return constructBool(internal.pos + 60, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 61, internal.ast);
  }

  get accessibility() {
    const internal = this.#internal;
    return constructOptionTSAccessibility(internal.pos + 62, internal.ast);
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugMethodDefinition.prototype);
  }
}

const DebugMethodDefinition = class MethodDefinition {};

function constructMethodDefinitionType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "MethodDefinition";
    case 1:
      return "TSAbstractMethodDefinition";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for MethodDefinitionType`);
  }
}

export class PropertyDefinition {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $decorators: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get type() {
    const internal = this.#internal;
    return constructPropertyDefinitionType(internal.pos + 72, internal.ast);
  }

  get decorators() {
    const internal = this.#internal,
      cached = internal.$decorators;
    if (cached !== void 0) return cached;
    return (internal.$decorators = constructVecDecorator(internal.pos + 8, internal.ast));
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 32, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 48, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 56, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 73, internal.ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.pos + 74, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 75, internal.ast);
  }

  get override() {
    const internal = this.#internal;
    return constructBool(internal.pos + 76, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 77, internal.ast);
  }

  get definite() {
    const internal = this.#internal;
    return constructBool(internal.pos + 78, internal.ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructBool(internal.pos + 79, internal.ast);
  }

  get accessibility() {
    const internal = this.#internal;
    return constructOptionTSAccessibility(internal.pos + 80, internal.ast);
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugPropertyDefinition.prototype);
  }
}

const DebugPropertyDefinition = class PropertyDefinition {};

function constructPropertyDefinitionType(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "PropertyDefinition";
    case 1:
      return "TSAbstractPropertyDefinition";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyDefinitionType`);
  }
}

function constructMethodDefinitionKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "constructor";
    case 1:
      return "method";
    case 2:
      return "get";
    case 3:
      return "set";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for MethodDefinitionKind`);
  }
}

export class PrivateIdentifier {
  type = "PrivateIdentifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "PrivateIdentifier",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugPrivateIdentifier.prototype);
  }
}

const DebugPrivateIdentifier = class PrivateIdentifier {};

export class StaticBlock {
  type = "StaticBlock";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecStatement(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "StaticBlock",
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugStaticBlock.prototype);
  }
}

const DebugStaticBlock = class StaticBlock {};

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
      return "AccessorProperty";
    case 1:
      return "TSAbstractAccessorProperty";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AccessorPropertyType`);
  }
}

export class AccessorProperty {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $decorators: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get type() {
    const internal = this.#internal;
    return constructAccessorPropertyType(internal.pos + 72, internal.ast);
  }

  get decorators() {
    const internal = this.#internal,
      cached = internal.$decorators;
    if (cached !== void 0) return cached;
    return (internal.$decorators = constructVecDecorator(internal.pos + 8, internal.ast));
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 32, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 48, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 56, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 73, internal.ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.pos + 74, internal.ast);
  }

  get override() {
    const internal = this.#internal;
    return constructBool(internal.pos + 75, internal.ast);
  }

  get definite() {
    const internal = this.#internal;
    return constructBool(internal.pos + 76, internal.ast);
  }

  get accessibility() {
    const internal = this.#internal;
    return constructOptionTSAccessibility(internal.pos + 77, internal.ast);
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugAccessorProperty.prototype);
  }
}

const DebugAccessorProperty = class AccessorProperty {};

export class ImportExpression {
  type = "ImportExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get source() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get options() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 24, internal.ast);
  }

  get phase() {
    const internal = this.#internal;
    return constructOptionImportPhase(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "ImportExpression",
      start: this.start,
      end: this.end,
      source: this.source,
      options: this.options,
      phase: this.phase,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportExpression.prototype);
  }
}

const DebugImportExpression = class ImportExpression {};

export class ImportDeclaration {
  type = "ImportDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $specifiers: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get specifiers() {
    const internal = this.#internal,
      cached = internal.$specifiers;
    if (cached !== void 0) return cached;
    return (internal.$specifiers = constructOptionVecImportDeclarationSpecifier(
      internal.pos + 8,
      internal.ast,
    ));
  }

  get source() {
    const internal = this.#internal;
    return new StringLiteral(internal.pos + 32, internal.ast);
  }

  get phase() {
    const internal = this.#internal;
    return constructOptionImportPhase(internal.pos + 88, internal.ast);
  }

  get attributes() {
    const internal = this.#internal;
    return constructOptionBoxWithClause(internal.pos + 80, internal.ast);
  }

  get importKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.pos + 89, internal.ast);
  }

  toJSON() {
    return {
      type: "ImportDeclaration",
      start: this.start,
      end: this.end,
      specifiers: this.specifiers,
      source: this.source,
      phase: this.phase,
      attributes: this.attributes,
      importKind: this.importKind,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportDeclaration.prototype);
  }
}

const DebugImportDeclaration = class ImportDeclaration {};

function constructImportPhase(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "source";
    case 1:
      return "defer";
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

export class ImportSpecifier {
  type = "ImportSpecifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get imported() {
    const internal = this.#internal;
    return constructModuleExportName(internal.pos + 8, internal.ast);
  }

  get local() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 64, internal.ast);
  }

  get importKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.pos + 96, internal.ast);
  }

  toJSON() {
    return {
      type: "ImportSpecifier",
      start: this.start,
      end: this.end,
      imported: this.imported,
      local: this.local,
      importKind: this.importKind,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportSpecifier.prototype);
  }
}

const DebugImportSpecifier = class ImportSpecifier {};

export class ImportDefaultSpecifier {
  type = "ImportDefaultSpecifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get local() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ImportDefaultSpecifier",
      start: this.start,
      end: this.end,
      local: this.local,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportDefaultSpecifier.prototype);
  }
}

const DebugImportDefaultSpecifier = class ImportDefaultSpecifier {};

export class ImportNamespaceSpecifier {
  type = "ImportNamespaceSpecifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get local() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ImportNamespaceSpecifier",
      start: this.start,
      end: this.end,
      local: this.local,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportNamespaceSpecifier.prototype);
  }
}

const DebugImportNamespaceSpecifier = class ImportNamespaceSpecifier {};

export class WithClause {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $attributes: void 0 };
    nodes.set(pos, this);
  }

  get attributes() {
    const internal = this.#internal,
      cached = internal.$attributes;
    if (cached !== void 0) return cached;
    return (internal.$attributes = constructVecImportAttribute(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      attributes: this.attributes,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugWithClause.prototype);
  }
}

const DebugWithClause = class WithClause {};

export class ImportAttribute {
  type = "ImportAttribute";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return constructImportAttributeKey(internal.pos + 8, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return new StringLiteral(internal.pos + 64, internal.ast);
  }

  toJSON() {
    return {
      type: "ImportAttribute",
      start: this.start,
      end: this.end,
      key: this.key,
      value: this.value,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportAttribute.prototype);
  }
}

const DebugImportAttribute = class ImportAttribute {};

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

export class ExportNamedDeclaration {
  type = "ExportNamedDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $specifiers: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get declaration() {
    const internal = this.#internal;
    return constructOptionDeclaration(internal.pos + 8, internal.ast);
  }

  get specifiers() {
    const internal = this.#internal,
      cached = internal.$specifiers;
    if (cached !== void 0) return cached;
    return (internal.$specifiers = constructVecExportSpecifier(internal.pos + 24, internal.ast));
  }

  get source() {
    const internal = this.#internal;
    return constructOptionStringLiteral(internal.pos + 48, internal.ast);
  }

  get exportKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.pos + 104, internal.ast);
  }

  get attributes() {
    const internal = this.#internal;
    return constructOptionBoxWithClause(internal.pos + 96, internal.ast);
  }

  toJSON() {
    return {
      type: "ExportNamedDeclaration",
      start: this.start,
      end: this.end,
      declaration: this.declaration,
      specifiers: this.specifiers,
      source: this.source,
      exportKind: this.exportKind,
      attributes: this.attributes,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugExportNamedDeclaration.prototype);
  }
}

const DebugExportNamedDeclaration = class ExportNamedDeclaration {};

export class ExportDefaultDeclaration {
  type = "ExportDefaultDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get declaration() {
    const internal = this.#internal;
    return constructExportDefaultDeclarationKind(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "ExportDefaultDeclaration",
      start: this.start,
      end: this.end,
      declaration: this.declaration,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugExportDefaultDeclaration.prototype);
  }
}

const DebugExportDefaultDeclaration = class ExportDefaultDeclaration {};

export class ExportAllDeclaration {
  type = "ExportAllDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get exported() {
    const internal = this.#internal;
    return constructOptionModuleExportName(internal.pos + 8, internal.ast);
  }

  get source() {
    const internal = this.#internal;
    return new StringLiteral(internal.pos + 64, internal.ast);
  }

  get attributes() {
    const internal = this.#internal;
    return constructOptionBoxWithClause(internal.pos + 112, internal.ast);
  }

  get exportKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.pos + 120, internal.ast);
  }

  toJSON() {
    return {
      type: "ExportAllDeclaration",
      start: this.start,
      end: this.end,
      exported: this.exported,
      source: this.source,
      attributes: this.attributes,
      exportKind: this.exportKind,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugExportAllDeclaration.prototype);
  }
}

const DebugExportAllDeclaration = class ExportAllDeclaration {};

export class ExportSpecifier {
  type = "ExportSpecifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get local() {
    const internal = this.#internal;
    return constructModuleExportName(internal.pos + 8, internal.ast);
  }

  get exported() {
    const internal = this.#internal;
    return constructModuleExportName(internal.pos + 64, internal.ast);
  }

  get exportKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.pos + 120, internal.ast);
  }

  toJSON() {
    return {
      type: "ExportSpecifier",
      start: this.start,
      end: this.end,
      local: this.local,
      exported: this.exported,
      exportKind: this.exportKind,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugExportSpecifier.prototype);
  }
}

const DebugExportSpecifier = class ExportSpecifier {};

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
      throw new Error(
        `Unexpected discriminant ${ast.buffer[pos]} for ExportDefaultDeclarationKind`,
      );
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

export class V8IntrinsicExpression {
  type = "V8IntrinsicExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $arguments: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 8, internal.ast);
  }

  get arguments() {
    const internal = this.#internal,
      cached = internal.$arguments;
    if (cached !== void 0) return cached;
    return (internal.$arguments = constructVecArgument(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "V8IntrinsicExpression",
      start: this.start,
      end: this.end,
      name: this.name,
      arguments: this.arguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugV8IntrinsicExpression.prototype);
  }
}

const DebugV8IntrinsicExpression = class V8IntrinsicExpression {};

export class BooleanLiteral {
  type = "BooleanLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructBool(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "BooleanLiteral",
      start: this.start,
      end: this.end,
      value: this.value,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBooleanLiteral.prototype);
  }
}

const DebugBooleanLiteral = class BooleanLiteral {};

export class NullLiteral {
  type = "NullLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "NullLiteral",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugNullLiteral.prototype);
  }
}

const DebugNullLiteral = class NullLiteral {};

export class NumericLiteral {
  type = "NumericLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $raw: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructF64(internal.pos + 8, internal.ast);
  }

  get raw() {
    const internal = this.#internal,
      cached = internal.$raw;
    if (cached !== void 0) return cached;
    return (internal.$raw = constructOptionStr(internal.pos + 16, internal.ast));
  }

  toJSON() {
    return {
      type: "NumericLiteral",
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugNumericLiteral.prototype);
  }
}

const DebugNumericLiteral = class NumericLiteral {};

export class StringLiteral {
  type = "StringLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $value: void 0, $raw: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal,
      cached = internal.$value;
    if (cached !== void 0) return cached;
    return (internal.$value = constructStr(internal.pos + 8, internal.ast));
  }

  get raw() {
    const internal = this.#internal,
      cached = internal.$raw;
    if (cached !== void 0) return cached;
    return (internal.$raw = constructOptionStr(internal.pos + 24, internal.ast));
  }

  toJSON() {
    return {
      type: "StringLiteral",
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugStringLiteral.prototype);
  }
}

const DebugStringLiteral = class StringLiteral {};

export class BigIntLiteral {
  type = "BigIntLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $value: void 0, $raw: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal,
      cached = internal.$value;
    if (cached !== void 0) return cached;
    return (internal.$value = constructStr(internal.pos + 8, internal.ast));
  }

  get raw() {
    const internal = this.#internal,
      cached = internal.$raw;
    if (cached !== void 0) return cached;
    return (internal.$raw = constructOptionStr(internal.pos + 24, internal.ast));
  }

  toJSON() {
    return {
      type: "BigIntLiteral",
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugBigIntLiteral.prototype);
  }
}

const DebugBigIntLiteral = class BigIntLiteral {};

export class RegExpLiteral {
  type = "RegExpLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $raw: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get regex() {
    const internal = this.#internal;
    return new RegExp(internal.pos + 8, internal.ast);
  }

  get raw() {
    const internal = this.#internal,
      cached = internal.$raw;
    if (cached !== void 0) return cached;
    return (internal.$raw = constructOptionStr(internal.pos + 40, internal.ast));
  }

  toJSON() {
    return {
      type: "RegExpLiteral",
      start: this.start,
      end: this.end,
      regex: this.regex,
      raw: this.raw,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugRegExpLiteral.prototype);
  }
}

const DebugRegExpLiteral = class RegExpLiteral {};

export class RegExp {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    // `pos` would be same as one of fields, so add offset to ensure unique cache key
    const cached = nodes.get(pos + 1);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos + 1, this);
  }

  get pattern() {
    const internal = this.#internal;
    return new RegExpPattern(internal.pos, internal.ast);
  }

  get flags() {
    const internal = this.#internal;
    return new RegExpFlags(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      pattern: this.pattern,
      flags: this.flags,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugRegExp.prototype);
  }
}

const DebugRegExp = class RegExp {};

export class RegExpPattern {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $pattern: void 0 };
    nodes.set(pos, this);
  }

  get pattern() {
    const internal = this.#internal,
      cached = internal.$pattern;
    if (cached !== void 0) return cached;
    return (internal.$pattern = constructStr(internal.pos, internal.ast));
  }

  toJSON() {
    return {
      pattern: this.pattern,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugRegExpPattern.prototype);
  }
}

const DebugRegExpPattern = class RegExpPattern {};

export class RegExpFlags {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  toJSON() {
    return {};
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugRegExpFlags.prototype);
  }
}

const DebugRegExpFlags = class RegExpFlags {};

export class JSXElement {
  type = "JSXElement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $children: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get openingElement() {
    const internal = this.#internal;
    return constructBoxJSXOpeningElement(internal.pos + 8, internal.ast);
  }

  get children() {
    const internal = this.#internal,
      cached = internal.$children;
    if (cached !== void 0) return cached;
    return (internal.$children = constructVecJSXChild(internal.pos + 16, internal.ast));
  }

  get closingElement() {
    const internal = this.#internal;
    return constructOptionBoxJSXClosingElement(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXElement",
      start: this.start,
      end: this.end,
      openingElement: this.openingElement,
      children: this.children,
      closingElement: this.closingElement,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXElement.prototype);
  }
}

const DebugJSXElement = class JSXElement {};

export class JSXOpeningElement {
  type = "JSXOpeningElement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $attributes: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal;
    return constructJSXElementName(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  get attributes() {
    const internal = this.#internal,
      cached = internal.$attributes;
    if (cached !== void 0) return cached;
    return (internal.$attributes = constructVecJSXAttributeItem(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "JSXOpeningElement",
      start: this.start,
      end: this.end,
      name: this.name,
      typeArguments: this.typeArguments,
      attributes: this.attributes,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXOpeningElement.prototype);
  }
}

const DebugJSXOpeningElement = class JSXOpeningElement {};

export class JSXClosingElement {
  type = "JSXClosingElement";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal;
    return constructJSXElementName(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXClosingElement",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXClosingElement.prototype);
  }
}

const DebugJSXClosingElement = class JSXClosingElement {};

export class JSXFragment {
  type = "JSXFragment";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $children: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get openingFragment() {
    const internal = this.#internal;
    return new JSXOpeningFragment(internal.pos + 8, internal.ast);
  }

  get children() {
    const internal = this.#internal,
      cached = internal.$children;
    if (cached !== void 0) return cached;
    return (internal.$children = constructVecJSXChild(internal.pos + 16, internal.ast));
  }

  get closingFragment() {
    const internal = this.#internal;
    return new JSXClosingFragment(internal.pos + 40, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXFragment",
      start: this.start,
      end: this.end,
      openingFragment: this.openingFragment,
      children: this.children,
      closingFragment: this.closingFragment,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXFragment.prototype);
  }
}

const DebugJSXFragment = class JSXFragment {};

export class JSXOpeningFragment {
  type = "JSXOpeningFragment";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXOpeningFragment",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXOpeningFragment.prototype);
  }
}

const DebugJSXOpeningFragment = class JSXOpeningFragment {};

export class JSXClosingFragment {
  type = "JSXClosingFragment";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXClosingFragment",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXClosingFragment.prototype);
  }
}

const DebugJSXClosingFragment = class JSXClosingFragment {};

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

export class JSXNamespacedName {
  type = "JSXNamespacedName";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get namespace() {
    const internal = this.#internal;
    return new JSXIdentifier(internal.pos + 8, internal.ast);
  }

  get name() {
    const internal = this.#internal;
    return new JSXIdentifier(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXNamespacedName",
      start: this.start,
      end: this.end,
      namespace: this.namespace,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXNamespacedName.prototype);
  }
}

const DebugJSXNamespacedName = class JSXNamespacedName {};

export class JSXMemberExpression {
  type = "JSXMemberExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get object() {
    const internal = this.#internal;
    return constructJSXMemberExpressionObject(internal.pos + 8, internal.ast);
  }

  get property() {
    const internal = this.#internal;
    return new JSXIdentifier(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXMemberExpression",
      start: this.start,
      end: this.end,
      object: this.object,
      property: this.property,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXMemberExpression.prototype);
  }
}

const DebugJSXMemberExpression = class JSXMemberExpression {};

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

export class JSXExpressionContainer {
  type = "JSXExpressionContainer";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructJSXExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXExpressionContainer",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXExpressionContainer.prototype);
  }
}

const DebugJSXExpressionContainer = class JSXExpressionContainer {};

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

export class JSXEmptyExpression {
  type = "JSXEmptyExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXEmptyExpression",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXEmptyExpression.prototype);
  }
}

const DebugJSXEmptyExpression = class JSXEmptyExpression {};

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

export class JSXAttribute {
  type = "JSXAttribute";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal;
    return constructJSXAttributeName(internal.pos + 8, internal.ast);
  }

  get value() {
    const internal = this.#internal;
    return constructOptionJSXAttributeValue(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXAttribute",
      start: this.start,
      end: this.end,
      name: this.name,
      value: this.value,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXAttribute.prototype);
  }
}

const DebugJSXAttribute = class JSXAttribute {};

export class JSXSpreadAttribute {
  type = "JSXSpreadAttribute";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get argument() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXSpreadAttribute",
      start: this.start,
      end: this.end,
      argument: this.argument,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXSpreadAttribute.prototype);
  }
}

const DebugJSXSpreadAttribute = class JSXSpreadAttribute {};

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

export class JSXIdentifier {
  type = "JSXIdentifier";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "JSXIdentifier",
      start: this.start,
      end: this.end,
      name: this.name,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXIdentifier.prototype);
  }
}

const DebugJSXIdentifier = class JSXIdentifier {};

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

export class JSXSpreadChild {
  type = "JSXSpreadChild";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "JSXSpreadChild",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXSpreadChild.prototype);
  }
}

const DebugJSXSpreadChild = class JSXSpreadChild {};

export class JSXText {
  type = "JSXText";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $value: void 0, $raw: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get value() {
    const internal = this.#internal,
      cached = internal.$value;
    if (cached !== void 0) return cached;
    return (internal.$value = constructStr(internal.pos + 8, internal.ast));
  }

  get raw() {
    const internal = this.#internal,
      cached = internal.$raw;
    if (cached !== void 0) return cached;
    return (internal.$raw = constructOptionStr(internal.pos + 24, internal.ast));
  }

  toJSON() {
    return {
      type: "JSXText",
      start: this.start,
      end: this.end,
      value: this.value,
      raw: this.raw,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSXText.prototype);
  }
}

const DebugJSXText = class JSXText {};

export class TSThisParameter {
  type = "TSThisParameter";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 16, internal.ast);
  }

  toJSON() {
    return {
      type: "TSThisParameter",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSThisParameter.prototype);
  }
}

const DebugTSThisParameter = class TSThisParameter {};

export class TSEnumDeclaration {
  type = "TSEnumDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return new TSEnumBody(internal.pos + 40, internal.ast);
  }

  get const() {
    const internal = this.#internal;
    return constructBool(internal.pos + 76, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 77, internal.ast);
  }

  toJSON() {
    return {
      type: "TSEnumDeclaration",
      start: this.start,
      end: this.end,
      id: this.id,
      body: this.body,
      const: this.const,
      declare: this.declare,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSEnumDeclaration.prototype);
  }
}

const DebugTSEnumDeclaration = class TSEnumDeclaration {};

export class TSEnumBody {
  type = "TSEnumBody";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $members: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get members() {
    const internal = this.#internal,
      cached = internal.$members;
    if (cached !== void 0) return cached;
    return (internal.$members = constructVecTSEnumMember(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSEnumBody",
      start: this.start,
      end: this.end,
      members: this.members,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSEnumBody.prototype);
  }
}

const DebugTSEnumBody = class TSEnumBody {};

export class TSEnumMember {
  type = "TSEnumMember";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return constructTSEnumMemberName(internal.pos + 8, internal.ast);
  }

  get initializer() {
    const internal = this.#internal;
    return constructOptionExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSEnumMember",
      start: this.start,
      end: this.end,
      id: this.id,
      initializer: this.initializer,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSEnumMember.prototype);
  }
}

const DebugTSEnumMember = class TSEnumMember {};

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

export class TSTypeAnnotation {
  type = "TSTypeAnnotation";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeAnnotation",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeAnnotation.prototype);
  }
}

const DebugTSTypeAnnotation = class TSTypeAnnotation {};

export class TSLiteralType {
  type = "TSLiteralType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get literal() {
    const internal = this.#internal;
    return constructTSLiteral(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSLiteralType",
      start: this.start,
      end: this.end,
      literal: this.literal,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSLiteralType.prototype);
  }
}

const DebugTSLiteralType = class TSLiteralType {};

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

export class TSConditionalType {
  type = "TSConditionalType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get checkType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  get extendsType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 24, internal.ast);
  }

  get trueType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 40, internal.ast);
  }

  get falseType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 56, internal.ast);
  }

  toJSON() {
    return {
      type: "TSConditionalType",
      start: this.start,
      end: this.end,
      checkType: this.checkType,
      extendsType: this.extendsType,
      trueType: this.trueType,
      falseType: this.falseType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSConditionalType.prototype);
  }
}

const DebugTSConditionalType = class TSConditionalType {};

export class TSUnionType {
  type = "TSUnionType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $types: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get types() {
    const internal = this.#internal,
      cached = internal.$types;
    if (cached !== void 0) return cached;
    return (internal.$types = constructVecTSType(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSUnionType",
      start: this.start,
      end: this.end,
      types: this.types,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSUnionType.prototype);
  }
}

const DebugTSUnionType = class TSUnionType {};

export class TSIntersectionType {
  type = "TSIntersectionType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $types: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get types() {
    const internal = this.#internal,
      cached = internal.$types;
    if (cached !== void 0) return cached;
    return (internal.$types = constructVecTSType(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSIntersectionType",
      start: this.start,
      end: this.end,
      types: this.types,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSIntersectionType.prototype);
  }
}

const DebugTSIntersectionType = class TSIntersectionType {};

export class TSParenthesizedType {
  type = "TSParenthesizedType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSParenthesizedType",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSParenthesizedType.prototype);
  }
}

const DebugTSParenthesizedType = class TSParenthesizedType {};

export class TSTypeOperator {
  type = "TSTypeOperator";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get operator() {
    const internal = this.#internal;
    return constructTSTypeOperatorOperator(internal.pos + 24, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeOperator",
      start: this.start,
      end: this.end,
      operator: this.operator,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeOperator.prototype);
  }
}

const DebugTSTypeOperator = class TSTypeOperator {};

function constructTSTypeOperatorOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "keyof";
    case 1:
      return "unique";
    case 2:
      return "readonly";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeOperatorOperator`);
  }
}

export class TSArrayType {
  type = "TSArrayType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get elementType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSArrayType",
      start: this.start,
      end: this.end,
      elementType: this.elementType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSArrayType.prototype);
  }
}

const DebugTSArrayType = class TSArrayType {};

export class TSIndexedAccessType {
  type = "TSIndexedAccessType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get objectType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  get indexType() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSIndexedAccessType",
      start: this.start,
      end: this.end,
      objectType: this.objectType,
      indexType: this.indexType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSIndexedAccessType.prototype);
  }
}

const DebugTSIndexedAccessType = class TSIndexedAccessType {};

export class TSTupleType {
  type = "TSTupleType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $elementTypes: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get elementTypes() {
    const internal = this.#internal,
      cached = internal.$elementTypes;
    if (cached !== void 0) return cached;
    return (internal.$elementTypes = constructVecTSTupleElement(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSTupleType",
      start: this.start,
      end: this.end,
      elementTypes: this.elementTypes,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTupleType.prototype);
  }
}

const DebugTSTupleType = class TSTupleType {};

export class TSNamedTupleMember {
  type = "TSNamedTupleMember";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get label() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 8, internal.ast);
  }

  get elementType() {
    const internal = this.#internal;
    return constructTSTupleElement(internal.pos + 32, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 48, internal.ast);
  }

  toJSON() {
    return {
      type: "TSNamedTupleMember",
      start: this.start,
      end: this.end,
      label: this.label,
      elementType: this.elementType,
      optional: this.optional,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSNamedTupleMember.prototype);
  }
}

const DebugTSNamedTupleMember = class TSNamedTupleMember {};

export class TSOptionalType {
  type = "TSOptionalType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSOptionalType",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSOptionalType.prototype);
  }
}

const DebugTSOptionalType = class TSOptionalType {};

export class TSRestType {
  type = "TSRestType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSRestType",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSRestType.prototype);
  }
}

const DebugTSRestType = class TSRestType {};

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

export class TSAnyKeyword {
  type = "TSAnyKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSAnyKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSAnyKeyword.prototype);
  }
}

const DebugTSAnyKeyword = class TSAnyKeyword {};

export class TSStringKeyword {
  type = "TSStringKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSStringKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSStringKeyword.prototype);
  }
}

const DebugTSStringKeyword = class TSStringKeyword {};

export class TSBooleanKeyword {
  type = "TSBooleanKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSBooleanKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSBooleanKeyword.prototype);
  }
}

const DebugTSBooleanKeyword = class TSBooleanKeyword {};

export class TSNumberKeyword {
  type = "TSNumberKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSNumberKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSNumberKeyword.prototype);
  }
}

const DebugTSNumberKeyword = class TSNumberKeyword {};

export class TSNeverKeyword {
  type = "TSNeverKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSNeverKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSNeverKeyword.prototype);
  }
}

const DebugTSNeverKeyword = class TSNeverKeyword {};

export class TSIntrinsicKeyword {
  type = "TSIntrinsicKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSIntrinsicKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSIntrinsicKeyword.prototype);
  }
}

const DebugTSIntrinsicKeyword = class TSIntrinsicKeyword {};

export class TSUnknownKeyword {
  type = "TSUnknownKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSUnknownKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSUnknownKeyword.prototype);
  }
}

const DebugTSUnknownKeyword = class TSUnknownKeyword {};

export class TSNullKeyword {
  type = "TSNullKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSNullKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSNullKeyword.prototype);
  }
}

const DebugTSNullKeyword = class TSNullKeyword {};

export class TSUndefinedKeyword {
  type = "TSUndefinedKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSUndefinedKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSUndefinedKeyword.prototype);
  }
}

const DebugTSUndefinedKeyword = class TSUndefinedKeyword {};

export class TSVoidKeyword {
  type = "TSVoidKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSVoidKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSVoidKeyword.prototype);
  }
}

const DebugTSVoidKeyword = class TSVoidKeyword {};

export class TSSymbolKeyword {
  type = "TSSymbolKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSSymbolKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSSymbolKeyword.prototype);
  }
}

const DebugTSSymbolKeyword = class TSSymbolKeyword {};

export class TSThisType {
  type = "TSThisType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSThisType",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSThisType.prototype);
  }
}

const DebugTSThisType = class TSThisType {};

export class TSObjectKeyword {
  type = "TSObjectKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSObjectKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSObjectKeyword.prototype);
  }
}

const DebugTSObjectKeyword = class TSObjectKeyword {};

export class TSBigIntKeyword {
  type = "TSBigIntKeyword";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "TSBigIntKeyword",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSBigIntKeyword.prototype);
  }
}

const DebugTSBigIntKeyword = class TSBigIntKeyword {};

export class TSTypeReference {
  type = "TSTypeReference";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeName() {
    const internal = this.#internal;
    return constructTSTypeName(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeReference",
      start: this.start,
      end: this.end,
      typeName: this.typeName,
      typeArguments: this.typeArguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeReference.prototype);
  }
}

const DebugTSTypeReference = class TSTypeReference {};

function constructTSTypeName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSQualifiedName(pos + 8, ast);
    case 2:
      return constructBoxThisExpression(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeName`);
  }
}

export class TSQualifiedName {
  type = "TSQualifiedName";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructTSTypeName(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSQualifiedName",
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSQualifiedName.prototype);
  }
}

const DebugTSQualifiedName = class TSQualifiedName {};

export class TSTypeParameterInstantiation {
  type = "TSTypeParameterInstantiation";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $params: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get params() {
    const internal = this.#internal,
      cached = internal.$params;
    if (cached !== void 0) return cached;
    return (internal.$params = constructVecTSType(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSTypeParameterInstantiation",
      start: this.start,
      end: this.end,
      params: this.params,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeParameterInstantiation.prototype);
  }
}

const DebugTSTypeParameterInstantiation = class TSTypeParameterInstantiation {};

export class TSTypeParameter {
  type = "TSTypeParameter";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  get constraint() {
    const internal = this.#internal;
    return constructOptionTSType(internal.pos + 40, internal.ast);
  }

  get default() {
    const internal = this.#internal;
    return constructOptionTSType(internal.pos + 56, internal.ast);
  }

  get in() {
    const internal = this.#internal;
    return constructBool(internal.pos + 72, internal.ast);
  }

  get out() {
    const internal = this.#internal;
    return constructBool(internal.pos + 73, internal.ast);
  }

  get const() {
    const internal = this.#internal;
    return constructBool(internal.pos + 74, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeParameter",
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeParameter.prototype);
  }
}

const DebugTSTypeParameter = class TSTypeParameter {};

export class TSTypeParameterDeclaration {
  type = "TSTypeParameterDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $params: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get params() {
    const internal = this.#internal,
      cached = internal.$params;
    if (cached !== void 0) return cached;
    return (internal.$params = constructVecTSTypeParameter(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSTypeParameterDeclaration",
      start: this.start,
      end: this.end,
      params: this.params,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeParameterDeclaration.prototype);
  }
}

const DebugTSTypeParameterDeclaration = class TSTypeParameterDeclaration {};

export class TSTypeAliasDeclaration {
  type = "TSTypeAliasDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 40, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 48, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 68, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeAliasDeclaration",
      start: this.start,
      end: this.end,
      id: this.id,
      typeParameters: this.typeParameters,
      typeAnnotation: this.typeAnnotation,
      declare: this.declare,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeAliasDeclaration.prototype);
  }
}

const DebugTSTypeAliasDeclaration = class TSTypeAliasDeclaration {};

function constructTSAccessibility(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "private";
    case 1:
      return "protected";
    case 2:
      return "public";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSAccessibility`);
  }
}

export class TSClassImplements {
  type = "TSClassImplements";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructTSTypeName(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSClassImplements",
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeArguments: this.typeArguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSClassImplements.prototype);
  }
}

const DebugTSClassImplements = class TSClassImplements {};

export class TSInterfaceDeclaration {
  type = "TSInterfaceDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $extends: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 40, internal.ast);
  }

  get extends() {
    const internal = this.#internal,
      cached = internal.$extends;
    if (cached !== void 0) return cached;
    return (internal.$extends = constructVecTSInterfaceHeritage(internal.pos + 48, internal.ast));
  }

  get body() {
    const internal = this.#internal;
    return constructBoxTSInterfaceBody(internal.pos + 72, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 84, internal.ast);
  }

  toJSON() {
    return {
      type: "TSInterfaceDeclaration",
      start: this.start,
      end: this.end,
      id: this.id,
      typeParameters: this.typeParameters,
      extends: this.extends,
      body: this.body,
      declare: this.declare,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSInterfaceDeclaration.prototype);
  }
}

const DebugTSInterfaceDeclaration = class TSInterfaceDeclaration {};

export class TSInterfaceBody {
  type = "TSInterfaceBody";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecTSSignature(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSInterfaceBody",
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSInterfaceBody.prototype);
  }
}

const DebugTSInterfaceBody = class TSInterfaceBody {};

export class TSPropertySignature {
  type = "TSPropertySignature";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 32, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 33, internal.ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructBool(internal.pos + 34, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 8, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSPropertySignature",
      start: this.start,
      end: this.end,
      computed: this.computed,
      optional: this.optional,
      readonly: this.readonly,
      key: this.key,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSPropertySignature.prototype);
  }
}

const DebugTSPropertySignature = class TSPropertySignature {};

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

export class TSIndexSignature {
  type = "TSIndexSignature";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $parameters: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get parameters() {
    const internal = this.#internal,
      cached = internal.$parameters;
    if (cached !== void 0) return cached;
    return (internal.$parameters = constructVecTSIndexSignatureName(
      internal.pos + 8,
      internal.ast,
    ));
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.pos + 32, internal.ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructBool(internal.pos + 40, internal.ast);
  }

  get static() {
    const internal = this.#internal;
    return constructBool(internal.pos + 41, internal.ast);
  }

  toJSON() {
    return {
      type: "TSIndexSignature",
      start: this.start,
      end: this.end,
      parameters: this.parameters,
      typeAnnotation: this.typeAnnotation,
      readonly: this.readonly,
      static: this.static,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSIndexSignature.prototype);
  }
}

const DebugTSIndexSignature = class TSIndexSignature {};

export class TSCallSignatureDeclaration {
  type = "TSCallSignatureDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 8, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 24, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "TSCallSignatureDeclaration",
      start: this.start,
      end: this.end,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSCallSignatureDeclaration.prototype);
  }
}

const DebugTSCallSignatureDeclaration = class TSCallSignatureDeclaration {};

function constructTSMethodSignatureKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "method";
    case 1:
      return "get";
    case 2:
      return "set";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSMethodSignatureKind`);
  }
}

export class TSMethodSignature {
  type = "TSMethodSignature";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get key() {
    const internal = this.#internal;
    return constructPropertyKey(internal.pos + 8, internal.ast);
  }

  get computed() {
    const internal = this.#internal;
    return constructBool(internal.pos + 60, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructBool(internal.pos + 61, internal.ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructTSMethodSignatureKind(internal.pos + 62, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 24, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 40, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 48, internal.ast);
  }

  toJSON() {
    return {
      type: "TSMethodSignature",
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSMethodSignature.prototype);
  }
}

const DebugTSMethodSignature = class TSMethodSignature {};

export class TSConstructSignatureDeclaration {
  type = "TSConstructSignatureDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 8, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 16, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSConstructSignatureDeclaration",
      start: this.start,
      end: this.end,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSConstructSignatureDeclaration.prototype);
  }
}

const DebugTSConstructSignatureDeclaration = class TSConstructSignatureDeclaration {};

export class TSIndexSignatureName {
  type = "TSIndexSignatureName";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $name: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get name() {
    const internal = this.#internal,
      cached = internal.$name;
    if (cached !== void 0) return cached;
    return (internal.$name = constructStr(internal.pos + 8, internal.ast));
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSIndexSignatureName",
      start: this.start,
      end: this.end,
      name: this.name,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSIndexSignatureName.prototype);
  }
}

const DebugTSIndexSignatureName = class TSIndexSignatureName {};

export class TSInterfaceHeritage {
  type = "TSInterfaceHeritage";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSInterfaceHeritage",
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeArguments: this.typeArguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSInterfaceHeritage.prototype);
  }
}

const DebugTSInterfaceHeritage = class TSInterfaceHeritage {};

export class TSTypePredicate {
  type = "TSTypePredicate";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get parameterName() {
    const internal = this.#internal;
    return constructTSTypePredicateName(internal.pos + 8, internal.ast);
  }

  get asserts() {
    const internal = this.#internal;
    return constructBool(internal.pos + 32, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeAnnotation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypePredicate",
      start: this.start,
      end: this.end,
      parameterName: this.parameterName,
      asserts: this.asserts,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypePredicate.prototype);
  }
}

const DebugTSTypePredicate = class TSTypePredicate {};

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

export class TSModuleDeclaration {
  type = "TSModuleDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return constructTSModuleDeclarationName(internal.pos + 8, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return constructOptionTSModuleDeclarationBody(internal.pos + 64, internal.ast);
  }

  get kind() {
    const internal = this.#internal;
    return constructTSModuleDeclarationKind(internal.pos + 84, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 85, internal.ast);
  }

  toJSON() {
    return {
      type: "TSModuleDeclaration",
      start: this.start,
      end: this.end,
      id: this.id,
      body: this.body,
      kind: this.kind,
      declare: this.declare,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSModuleDeclaration.prototype);
  }
}

const DebugTSModuleDeclaration = class TSModuleDeclaration {};

function constructTSModuleDeclarationKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "module";
    case 1:
      return "namespace";
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

export class TSGlobalDeclaration {
  type = "TSGlobalDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal;
    return new TSModuleBlock(internal.pos + 16, internal.ast);
  }

  get declare() {
    const internal = this.#internal;
    return constructBool(internal.pos + 76, internal.ast);
  }

  toJSON() {
    return {
      type: "TSGlobalDeclaration",
      start: this.start,
      end: this.end,
      body: this.body,
      declare: this.declare,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSGlobalDeclaration.prototype);
  }
}

const DebugTSGlobalDeclaration = class TSGlobalDeclaration {};

export class TSModuleBlock {
  type = "TSModuleBlock";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $body: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get body() {
    const internal = this.#internal,
      cached = internal.$body;
    if (cached !== void 0) return cached;
    return (internal.$body = constructVecStatement(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "TSModuleBlock",
      start: this.start,
      end: this.end,
      body: this.body,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSModuleBlock.prototype);
  }
}

const DebugTSModuleBlock = class TSModuleBlock {};

export class TSTypeLiteral {
  type = "TSTypeLiteral";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $members: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get members() {
    const internal = this.#internal,
      cached = internal.$members;
    if (cached !== void 0) return cached;
    return (internal.$members = constructVecTSSignature(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      type: "TSTypeLiteral",
      start: this.start,
      end: this.end,
      members: this.members,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeLiteral.prototype);
  }
}

const DebugTSTypeLiteral = class TSTypeLiteral {};

export class TSInferType {
  type = "TSInferType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeParameter() {
    const internal = this.#internal;
    return constructBoxTSTypeParameter(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSInferType",
      start: this.start,
      end: this.end,
      typeParameter: this.typeParameter,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSInferType.prototype);
  }
}

const DebugTSInferType = class TSInferType {};

export class TSTypeQuery {
  type = "TSTypeQuery";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get exprName() {
    const internal = this.#internal;
    return constructTSTypeQueryExprName(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeQuery",
      start: this.start,
      end: this.end,
      exprName: this.exprName,
      typeArguments: this.typeArguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeQuery.prototype);
  }
}

const DebugTSTypeQuery = class TSTypeQuery {};

function constructTSTypeQueryExprName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSQualifiedName(pos + 8, ast);
    case 2:
      return constructBoxThisExpression(pos + 8, ast);
    case 3:
      return constructBoxTSImportType(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeQueryExprName`);
  }
}

export class TSImportType {
  type = "TSImportType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get source() {
    const internal = this.#internal;
    return new StringLiteral(internal.pos + 8, internal.ast);
  }

  get options() {
    const internal = this.#internal;
    return constructOptionBoxObjectExpression(internal.pos + 56, internal.ast);
  }

  get qualifier() {
    const internal = this.#internal;
    return constructOptionTSImportTypeQualifier(internal.pos + 64, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterInstantiation(internal.pos + 80, internal.ast);
  }

  toJSON() {
    return {
      type: "TSImportType",
      start: this.start,
      end: this.end,
      source: this.source,
      options: this.options,
      qualifier: this.qualifier,
      typeArguments: this.typeArguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSImportType.prototype);
  }
}

const DebugTSImportType = class TSImportType {};

function constructTSImportTypeQualifier(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierName(pos + 8, ast);
    case 1:
      return constructBoxTSImportTypeQualifiedName(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSImportTypeQualifier`);
  }
}

export class TSImportTypeQualifiedName {
  type = "TSImportTypeQualifiedName";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get left() {
    const internal = this.#internal;
    return constructTSImportTypeQualifier(internal.pos + 8, internal.ast);
  }

  get right() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSImportTypeQualifiedName",
      start: this.start,
      end: this.end,
      left: this.left,
      right: this.right,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSImportTypeQualifiedName.prototype);
  }
}

const DebugTSImportTypeQualifiedName = class TSImportTypeQualifiedName {};

export class TSFunctionType {
  type = "TSFunctionType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 8, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 24, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.pos + 32, internal.ast);
  }

  toJSON() {
    return {
      type: "TSFunctionType",
      start: this.start,
      end: this.end,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSFunctionType.prototype);
  }
}

const DebugTSFunctionType = class TSFunctionType {};

export class TSConstructorType {
  type = "TSConstructorType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get abstract() {
    const internal = this.#internal;
    return constructBool(internal.pos + 36, internal.ast);
  }

  get typeParameters() {
    const internal = this.#internal;
    return constructOptionBoxTSTypeParameterDeclaration(internal.pos + 8, internal.ast);
  }

  get params() {
    const internal = this.#internal;
    return constructBoxFormalParameters(internal.pos + 16, internal.ast);
  }

  get returnType() {
    const internal = this.#internal;
    return constructBoxTSTypeAnnotation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSConstructorType",
      start: this.start,
      end: this.end,
      abstract: this.abstract,
      typeParameters: this.typeParameters,
      params: this.params,
      returnType: this.returnType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSConstructorType.prototype);
  }
}

const DebugTSConstructorType = class TSConstructorType {};

export class TSMappedType {
  type = "TSMappedType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get nameType() {
    const internal = this.#internal;
    return constructOptionTSType(internal.pos + 16, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructOptionTSType(internal.pos + 32, internal.ast);
  }

  get optional() {
    const internal = this.#internal;
    return constructOptionTSMappedTypeModifierOperator(internal.pos + 52, internal.ast);
  }

  get readonly() {
    const internal = this.#internal;
    return constructOptionTSMappedTypeModifierOperator(internal.pos + 53, internal.ast);
  }

  toJSON() {
    return {
      type: "TSMappedType",
      start: this.start,
      end: this.end,
      nameType: this.nameType,
      typeAnnotation: this.typeAnnotation,
      optional: this.optional,
      readonly: this.readonly,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSMappedType.prototype);
  }
}

const DebugTSMappedType = class TSMappedType {};

function constructTSMappedTypeModifierOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "true";
    case 1:
      return "+";
    case 2:
      return "-";
    default:
      throw new Error(
        `Unexpected discriminant ${ast.buffer[pos]} for TSMappedTypeModifierOperator`,
      );
  }
}

export class TSTemplateLiteralType {
  type = "TSTemplateLiteralType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $quasis: void 0, $types: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get quasis() {
    const internal = this.#internal,
      cached = internal.$quasis;
    if (cached !== void 0) return cached;
    return (internal.$quasis = constructVecTemplateElement(internal.pos + 8, internal.ast));
  }

  get types() {
    const internal = this.#internal,
      cached = internal.$types;
    if (cached !== void 0) return cached;
    return (internal.$types = constructVecTSType(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      type: "TSTemplateLiteralType",
      start: this.start,
      end: this.end,
      quasis: this.quasis,
      types: this.types,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTemplateLiteralType.prototype);
  }
}

const DebugTSTemplateLiteralType = class TSTemplateLiteralType {};

export class TSAsExpression {
  type = "TSAsExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSAsExpression",
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSAsExpression.prototype);
  }
}

const DebugTSAsExpression = class TSAsExpression {};

export class TSSatisfiesExpression {
  type = "TSSatisfiesExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSSatisfiesExpression",
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeAnnotation: this.typeAnnotation,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSSatisfiesExpression.prototype);
  }
}

const DebugTSSatisfiesExpression = class TSSatisfiesExpression {};

export class TSTypeAssertion {
  type = "TSTypeAssertion";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSTypeAssertion",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSTypeAssertion.prototype);
  }
}

const DebugTSTypeAssertion = class TSTypeAssertion {};

export class TSImportEqualsDeclaration {
  type = "TSImportEqualsDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return new BindingIdentifier(internal.pos + 8, internal.ast);
  }

  get moduleReference() {
    const internal = this.#internal;
    return constructTSModuleReference(internal.pos + 40, internal.ast);
  }

  get importKind() {
    const internal = this.#internal;
    return constructImportOrExportKind(internal.pos + 56, internal.ast);
  }

  toJSON() {
    return {
      type: "TSImportEqualsDeclaration",
      start: this.start,
      end: this.end,
      id: this.id,
      moduleReference: this.moduleReference,
      importKind: this.importKind,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSImportEqualsDeclaration.prototype);
  }
}

const DebugTSImportEqualsDeclaration = class TSImportEqualsDeclaration {};

function constructTSModuleReference(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return constructBoxIdentifierReference(pos + 8, ast);
    case 1:
      return constructBoxTSQualifiedName(pos + 8, ast);
    case 2:
      return constructBoxThisExpression(pos + 8, ast);
    case 3:
      return constructBoxTSExternalModuleReference(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleReference`);
  }
}

export class TSExternalModuleReference {
  type = "TSExternalModuleReference";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return new StringLiteral(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSExternalModuleReference",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSExternalModuleReference.prototype);
  }
}

const DebugTSExternalModuleReference = class TSExternalModuleReference {};

export class TSNonNullExpression {
  type = "TSNonNullExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSNonNullExpression",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSNonNullExpression.prototype);
  }
}

const DebugTSNonNullExpression = class TSNonNullExpression {};

export class Decorator {
  type = "Decorator";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "Decorator",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugDecorator.prototype);
  }
}

const DebugDecorator = class Decorator {};

export class TSExportAssignment {
  type = "TSExportAssignment";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSExportAssignment",
      start: this.start,
      end: this.end,
      expression: this.expression,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSExportAssignment.prototype);
  }
}

const DebugTSExportAssignment = class TSExportAssignment {};

export class TSNamespaceExportDeclaration {
  type = "TSNamespaceExportDeclaration";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get id() {
    const internal = this.#internal;
    return new IdentifierName(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      type: "TSNamespaceExportDeclaration",
      start: this.start,
      end: this.end,
      id: this.id,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSNamespaceExportDeclaration.prototype);
  }
}

const DebugTSNamespaceExportDeclaration = class TSNamespaceExportDeclaration {};

export class TSInstantiationExpression {
  type = "TSInstantiationExpression";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get expression() {
    const internal = this.#internal;
    return constructExpression(internal.pos + 8, internal.ast);
  }

  get typeArguments() {
    const internal = this.#internal;
    return constructBoxTSTypeParameterInstantiation(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "TSInstantiationExpression",
      start: this.start,
      end: this.end,
      expression: this.expression,
      typeArguments: this.typeArguments,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugTSInstantiationExpression.prototype);
  }
}

const DebugTSInstantiationExpression = class TSInstantiationExpression {};

function constructImportOrExportKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "value";
    case 1:
      return "type";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportOrExportKind`);
  }
}

export class JSDocNullableType {
  type = "JSDocNullableType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  get postfix() {
    const internal = this.#internal;
    return constructBool(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "JSDocNullableType",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
      postfix: this.postfix,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSDocNullableType.prototype);
  }
}

const DebugJSDocNullableType = class JSDocNullableType {};

export class JSDocNonNullableType {
  type = "JSDocNonNullableType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get typeAnnotation() {
    const internal = this.#internal;
    return constructTSType(internal.pos + 8, internal.ast);
  }

  get postfix() {
    const internal = this.#internal;
    return constructBool(internal.pos + 24, internal.ast);
  }

  toJSON() {
    return {
      type: "JSDocNonNullableType",
      start: this.start,
      end: this.end,
      typeAnnotation: this.typeAnnotation,
      postfix: this.postfix,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSDocNonNullableType.prototype);
  }
}

const DebugJSDocNonNullableType = class JSDocNonNullableType {};

export class JSDocUnknownType {
  type = "JSDocUnknownType";
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      type: "JSDocUnknownType",
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugJSDocUnknownType.prototype);
  }
}

const DebugJSDocUnknownType = class JSDocUnknownType {};

function constructCommentKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "Line";
    case 1:
      return "Block";
    case 2:
      return "Block";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for CommentKind`);
  }
}

export class Comment {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get type() {
    const internal = this.#internal;
    return constructCommentKind(internal.pos + 12, internal.ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      type: this.type,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugComment.prototype);
  }
}

const DebugComment = class Comment {};

export class NameSpan {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $value: void 0 };
    nodes.set(pos, this);
  }

  get value() {
    const internal = this.#internal,
      cached = internal.$value;
    if (cached !== void 0) return cached;
    return (internal.$value = constructStr(internal.pos + 8, internal.ast));
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      value: this.value,
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugNameSpan.prototype);
  }
}

const DebugNameSpan = class NameSpan {};

export class ImportEntry {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get importName() {
    const internal = this.#internal;
    return constructImportImportName(internal.pos + 32, internal.ast);
  }

  get localName() {
    const internal = this.#internal;
    return new NameSpan(internal.pos + 64, internal.ast);
  }

  get isType() {
    const internal = this.#internal;
    return constructBool(internal.pos + 88, internal.ast);
  }

  toJSON() {
    return {
      importName: this.importName,
      localName: this.localName,
      isType: this.isType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugImportEntry.prototype);
  }
}

const DebugImportEntry = class ImportEntry {};

function constructImportImportName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new NameSpan(pos + 8, ast);
    case 1:
      return "namespaceObject";
    case 2:
      return new Span(pos + 8, ast);
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportImportName`);
  }
}

export class ExportEntry {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get moduleRequest() {
    const internal = this.#internal;
    return constructOptionNameSpan(internal.pos + 16, internal.ast);
  }

  get importName() {
    const internal = this.#internal;
    return constructExportImportName(internal.pos + 40, internal.ast);
  }

  get exportName() {
    const internal = this.#internal;
    return constructExportExportName(internal.pos + 72, internal.ast);
  }

  get localName() {
    const internal = this.#internal;
    return constructExportLocalName(internal.pos + 104, internal.ast);
  }

  get isType() {
    const internal = this.#internal;
    return constructBool(internal.pos + 136, internal.ast);
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugExportEntry.prototype);
  }
}

const DebugExportEntry = class ExportEntry {};

function constructExportImportName(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return new NameSpan(pos + 8, ast);
    case 1:
      return "all";
    case 2:
      return "allButDefault";
    case 3:
      return "null";
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
      return "null";
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
      return "null";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ExportLocalName`);
  }
}

export class DynamicImport {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get moduleRequest() {
    const internal = this.#internal;
    return new Span(internal.pos + 8, internal.ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      moduleRequest: this.moduleRequest,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugDynamicImport.prototype);
  }
}

const DebugDynamicImport = class DynamicImport {};

function constructAssignmentOperator(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentOperator`);
  }
}

function constructBinaryOperator(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for BinaryOperator`);
  }
}

function constructLogicalOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "||";
    case 1:
      return "&&";
    case 2:
      return "??";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for LogicalOperator`);
  }
}

function constructUnaryOperator(pos, ast) {
  switch (ast.buffer[pos]) {
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
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for UnaryOperator`);
  }
}

function constructUpdateOperator(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "++";
    case 1:
      return "--";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for UpdateOperator`);
  }
}

export class Span {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSpan.prototype);
  }
}

const DebugSpan = class Span {};

export class SourceType {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast };
    nodes.set(pos, this);
  }

  get sourceType() {
    const internal = this.#internal;
    return constructModuleKind(internal.pos + 1, internal.ast);
  }

  toJSON() {
    return {
      sourceType: this.sourceType,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugSourceType.prototype);
  }
}

const DebugSourceType = class SourceType {};

function constructModuleKind(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "script";
    case 1:
      return "module";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ModuleKind`);
  }
}

export class RawTransferData {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    // `pos` would be same as one of fields, so add offset to ensure unique cache key
    const cached = nodes.get(pos + 1);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $comments: void 0, $errors: void 0 };
    nodes.set(pos + 1, this);
  }

  get program() {
    const internal = this.#internal;
    return new Program(internal.pos, internal.ast);
  }

  get comments() {
    const internal = this.#internal,
      cached = internal.$comments;
    if (cached !== void 0) return cached;
    return (internal.$comments = constructVecComment(internal.pos + 128, internal.ast));
  }

  get module() {
    const internal = this.#internal;
    return new EcmaScriptModule(internal.pos + 152, internal.ast);
  }

  get errors() {
    const internal = this.#internal,
      cached = internal.$errors;
    if (cached !== void 0) return cached;
    return (internal.$errors = constructVecError(internal.pos + 256, internal.ast));
  }

  toJSON() {
    return {
      program: this.program,
      comments: this.comments,
      module: this.module,
      errors: this.errors,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugRawTransferData.prototype);
  }
}

const DebugRawTransferData = class RawTransferData {};

export class Error {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = {
      pos,
      ast,
      $message: void 0,
      $labels: void 0,
      $helpMessage: void 0,
      $codeframe: void 0,
    };
    nodes.set(pos, this);
  }

  get severity() {
    const internal = this.#internal;
    return constructErrorSeverity(internal.pos + 72, internal.ast);
  }

  get message() {
    const internal = this.#internal,
      cached = internal.$message;
    if (cached !== void 0) return cached;
    return (internal.$message = constructStr(internal.pos, internal.ast));
  }

  get labels() {
    const internal = this.#internal,
      cached = internal.$labels;
    if (cached !== void 0) return cached;
    return (internal.$labels = constructVecErrorLabel(internal.pos + 16, internal.ast));
  }

  get helpMessage() {
    const internal = this.#internal,
      cached = internal.$helpMessage;
    if (cached !== void 0) return cached;
    return (internal.$helpMessage = constructOptionStr(internal.pos + 40, internal.ast));
  }

  get codeframe() {
    const internal = this.#internal,
      cached = internal.$codeframe;
    if (cached !== void 0) return cached;
    return (internal.$codeframe = constructStr(internal.pos + 56, internal.ast));
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugError.prototype);
  }
}

const DebugError = class Error {};

function constructErrorSeverity(pos, ast) {
  switch (ast.buffer[pos]) {
    case 0:
      return "Error";
    case 1:
      return "Warning";
    case 2:
      return "Advice";
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ErrorSeverity`);
  }
}

export class ErrorLabel {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $message: void 0 };
    nodes.set(pos, this);
  }

  get message() {
    const internal = this.#internal,
      cached = internal.$message;
    if (cached !== void 0) return cached;
    return (internal.$message = constructOptionStr(internal.pos + 8, internal.ast));
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  toJSON() {
    return {
      message: this.message,
      start: this.start,
      end: this.end,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugErrorLabel.prototype);
  }
}

const DebugErrorLabel = class ErrorLabel {};

export class EcmaScriptModule {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = {
      pos,
      ast,
      $staticImports: void 0,
      $staticExports: void 0,
      $dynamicImports: void 0,
      $importMetas: void 0,
    };
    nodes.set(pos, this);
  }

  get hasModuleSyntax() {
    const internal = this.#internal;
    return constructBool(internal.pos + 96, internal.ast);
  }

  get staticImports() {
    const internal = this.#internal,
      cached = internal.$staticImports;
    if (cached !== void 0) return cached;
    return (internal.$staticImports = constructVecStaticImport(internal.pos, internal.ast));
  }

  get staticExports() {
    const internal = this.#internal,
      cached = internal.$staticExports;
    if (cached !== void 0) return cached;
    return (internal.$staticExports = constructVecStaticExport(internal.pos + 24, internal.ast));
  }

  get dynamicImports() {
    const internal = this.#internal,
      cached = internal.$dynamicImports;
    if (cached !== void 0) return cached;
    return (internal.$dynamicImports = constructVecDynamicImport(internal.pos + 48, internal.ast));
  }

  get importMetas() {
    const internal = this.#internal,
      cached = internal.$importMetas;
    if (cached !== void 0) return cached;
    return (internal.$importMetas = constructVecSpan(internal.pos + 72, internal.ast));
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

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugEcmaScriptModule.prototype);
  }
}

const DebugEcmaScriptModule = class EcmaScriptModule {};

export class StaticImport {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $entries: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get moduleRequest() {
    const internal = this.#internal;
    return new NameSpan(internal.pos + 8, internal.ast);
  }

  get entries() {
    const internal = this.#internal,
      cached = internal.$entries;
    if (cached !== void 0) return cached;
    return (internal.$entries = constructVecImportEntry(internal.pos + 32, internal.ast));
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      moduleRequest: this.moduleRequest,
      entries: this.entries,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugStaticImport.prototype);
  }
}

const DebugStaticImport = class StaticImport {};

export class StaticExport {
  #internal;

  constructor(pos, ast) {
    if (ast?.token !== TOKEN) constructorError();

    const { nodes } = ast;
    const cached = nodes.get(pos);
    if (cached !== void 0) return cached;

    this.#internal = { pos, ast, $entries: void 0 };
    nodes.set(pos, this);
  }

  get start() {
    const internal = this.#internal;
    return constructU32(internal.pos, internal.ast);
  }

  get end() {
    const internal = this.#internal;
    return constructU32(internal.pos + 4, internal.ast);
  }

  get entries() {
    const internal = this.#internal,
      cached = internal.$entries;
    if (cached !== void 0) return cached;
    return (internal.$entries = constructVecExportEntry(internal.pos + 8, internal.ast));
  }

  toJSON() {
    return {
      start: this.start,
      end: this.end,
      entries: this.entries,
    };
  }

  [inspectSymbol]() {
    return Object.setPrototypeOf(this.toJSON(), DebugStaticExport.prototype);
  }
}

const DebugStaticExport = class StaticExport {};

function constructU32(pos, ast) {
  return ast.buffer.uint32[pos >> 2];
}

function constructU8(pos, ast) {
  return ast.buffer[pos];
}

function constructStr(pos, ast) {
  const pos32 = pos >> 2,
    { buffer } = ast,
    { uint32 } = buffer,
    len = uint32[pos32 + 2];
  if (len === 0) return "";

  pos = uint32[pos32];
  if (ast.sourceIsAscii && pos < ast.sourceByteLen) return ast.sourceText.substr(pos, len);

  // Longer strings use `TextDecoder`
  // TODO: Find best switch-over point
  const end = pos + len;
  if (len > 50) return decodeStr(buffer.subarray(pos, end));

  // Shorter strings decode by hand to avoid native call
  let out = "",
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructComment, ast);
}

function constructComment(pos, ast) {
  return new Comment(pos, ast);
}

function constructOptionHashbang(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)
    return null;
  return new Hashbang(pos, ast);
}

function constructVecDirective(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 72, constructDirective, ast);
}

function constructDirective(pos, ast) {
  return new Directive(pos, ast);
}

function constructVecStatement(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructStatement, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructArrayExpressionElement, ast);
}

function constructBoxSpreadElement(pos, ast) {
  return new SpreadElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecObjectPropertyKind(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructObjectPropertyKind, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 48, constructTemplateElement, ast);
}

function constructTemplateElement(pos, ast) {
  return new TemplateElement(pos, ast);
}

function constructVecExpression(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructExpression, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructArgument, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(
    uint32[pos32],
    uint32[pos32 + 2],
    16,
    constructOptionAssignmentTargetMaybeDefault,
    ast,
  );
}

function constructBoxAssignmentTargetRest(pos, ast) {
  return new AssignmentTargetRest(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionBoxAssignmentTargetRest(pos, ast) {
  if (ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0) return null;
  return constructBoxAssignmentTargetRest(pos, ast);
}

function constructVecAssignmentTargetProperty(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(
    uint32[pos32],
    uint32[pos32 + 2],
    16,
    constructAssignmentTargetProperty,
    ast,
  );
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

function constructBoxTSGlobalDeclaration(pos, ast) {
  return new TSGlobalDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructBoxTSImportEqualsDeclaration(pos, ast) {
  return new TSImportEqualsDeclaration(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecVariableDeclarator(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 64, constructVariableDeclarator, ast);
}

function constructVariableDeclarator(pos, ast) {
  return new VariableDeclarator(pos, ast);
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
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)
    return null;
  return new LabelIdentifier(pos, ast);
}

function constructVecSwitchCase(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 48, constructSwitchCase, ast);
}

function constructSwitchCase(pos, ast) {
  return new SwitchCase(pos, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 64, constructBindingProperty, ast);
}

function constructBindingProperty(pos, ast) {
  return new BindingProperty(pos, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 32, constructOptionBindingPattern, ast);
}

function constructOptionBindingIdentifier(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)
    return null;
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 72, constructFormalParameter, ast);
}

function constructFormalParameter(pos, ast) {
  return new FormalParameter(pos, ast);
}

function constructVecDecorator(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 24, constructDecorator, ast);
}

function constructDecorator(pos, ast) {
  return new Decorator(pos, ast);
}

function constructOptionTSAccessibility(pos, ast) {
  if (ast.buffer[pos] === 3) return null;
  return constructTSAccessibility(pos, ast);
}

function constructVecTSClassImplements(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 32, constructTSClassImplements, ast);
}

function constructTSClassImplements(pos, ast) {
  return new TSClassImplements(pos, ast);
}

function constructBoxClassBody(pos, ast) {
  return new ClassBody(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecClassElement(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructClassElement, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(
    uint32[pos32],
    uint32[pos32 + 2],
    16,
    constructImportDeclarationSpecifier,
    ast,
  );
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 112, constructImportAttribute, ast);
}

function constructImportAttribute(pos, ast) {
  return new ImportAttribute(pos, ast);
}

function constructOptionDeclaration(pos, ast) {
  if (ast.buffer[pos] === 31) return null;
  return constructDeclaration(pos, ast);
}

function constructVecExportSpecifier(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 128, constructExportSpecifier, ast);
}

function constructExportSpecifier(pos, ast) {
  return new ExportSpecifier(pos, ast);
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

function constructBoxJSXOpeningElement(pos, ast) {
  return new JSXOpeningElement(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecJSXChild(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructJSXChild, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructJSXAttributeItem, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 40, constructTSEnumMember, ast);
}

function constructTSEnumMember(pos, ast) {
  return new TSEnumMember(pos, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructTSType, ast);
}

function constructVecTSTupleElement(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructTSTupleElement, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 80, constructTSTypeParameter, ast);
}

function constructTSTypeParameter(pos, ast) {
  return new TSTypeParameter(pos, ast);
}

function constructVecTSInterfaceHeritage(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 32, constructTSInterfaceHeritage, ast);
}

function constructTSInterfaceHeritage(pos, ast) {
  return new TSInterfaceHeritage(pos, ast);
}

function constructBoxTSInterfaceBody(pos, ast) {
  return new TSInterfaceBody(ast.buffer.uint32[pos >> 2], ast);
}

function constructVecTSSignature(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructTSSignature, ast);
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
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 32, constructTSIndexSignatureName, ast);
}

function constructTSIndexSignatureName(pos, ast) {
  return new TSIndexSignatureName(pos, ast);
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

function constructOptionTSImportTypeQualifier(pos, ast) {
  if (ast.buffer[pos] === 2) return null;
  return constructTSImportTypeQualifier(pos, ast);
}

function constructBoxTSImportTypeQualifiedName(pos, ast) {
  return new TSImportTypeQualifiedName(ast.buffer.uint32[pos >> 2], ast);
}

function constructOptionTSMappedTypeModifierOperator(pos, ast) {
  if (ast.buffer[pos] === 3) return null;
  return constructTSMappedTypeModifierOperator(pos, ast);
}

function constructBoxTSExternalModuleReference(pos, ast) {
  return new TSExternalModuleReference(ast.buffer.uint32[pos >> 2], ast);
}

function constructU64(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return uint32[pos32] + uint32[pos32 + 1] * 4294967296;
}

function constructOptionNameSpan(pos, ast) {
  if (ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)
    return null;
  return new NameSpan(pos, ast);
}

function constructOptionU64(pos, ast) {
  if (ast.buffer[pos] === 0) return null;
  return constructU64(pos + 8, ast);
}

function constructVecError(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 80, constructError, ast);
}

function constructError(pos, ast) {
  return new Error(pos, ast);
}

function constructVecErrorLabel(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 24, constructErrorLabel, ast);
}

function constructErrorLabel(pos, ast) {
  return new ErrorLabel(pos, ast);
}

function constructVecStaticImport(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 56, constructStaticImport, ast);
}

function constructStaticImport(pos, ast) {
  return new StaticImport(pos, ast);
}

function constructVecStaticExport(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 32, constructStaticExport, ast);
}

function constructStaticExport(pos, ast) {
  return new StaticExport(pos, ast);
}

function constructVecDynamicImport(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 16, constructDynamicImport, ast);
}

function constructDynamicImport(pos, ast) {
  return new DynamicImport(pos, ast);
}

function constructVecSpan(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 8, constructSpan, ast);
}

function constructSpan(pos, ast) {
  return new Span(pos, ast);
}

function constructVecImportEntry(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 96, constructImportEntry, ast);
}

function constructImportEntry(pos, ast) {
  return new ImportEntry(pos, ast);
}

function constructVecExportEntry(pos, ast) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  return new NodeArray(uint32[pos32], uint32[pos32 + 2], 144, constructExportEntry, ast);
}

function constructExportEntry(pos, ast) {
  return new ExportEntry(pos, ast);
}
