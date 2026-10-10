let conversions = 0;
let evaluations = 0;
const key = { [Symbol.toPrimitive]() { conversions++; return "method"; } };
function getKey() { evaluations++; return key; }
let fieldName = "before";
const symbol = Symbol("symbol");
const unnamed = Symbol();
class A {
  static { fieldName = "after"; }
  static [fieldName] = function () {};
  static {}
  static [getKey()] = () => {};
  static {}
  static [symbol] = function () {};
  static {}
  static [unnamed] = () => {};
  static {}
  static #privateFunction = function () {};
  static {}
  static #privateArrow = () => {};
  static {}
  static 123 = function () {};
  static {}
  static getNames() { return [this.#privateFunction.name, this.#privateArrow.name]; }
}
expect(A.before.name).toBe("before");
expect(A.method.name).toBe("method");
expect([evaluations, conversions]).toEqual([1, 1]);
expect(A[symbol].name).toBe("[symbol]");
expect(A[unnamed].name).toBe("");
expect(A.getNames()).toEqual(["#privateFunction", "#privateArrow"]);
expect(A[123].name).toBe("123");
let objectConversions = 0;
const objectKey = { toString() { objectConversions++; return "ObjectClass"; } };
const object = { [objectKey]: class { static { this.seen = this.name; } } };
expect(object.ObjectClass.seen).toBe("ObjectClass");
expect(objectConversions).toBe(1);

for (const [key, name] of [[Symbol(), ""], [Symbol(""), "[]"], [Symbol("symbol"), "[symbol]"]]) {
  class FunctionField {
    static {}
    static [key] = function () {};
  }
  class ArrowField {
    static [key] = () => {};
    static {}
  }
  expect(FunctionField[key].name).toBe(name);
  expect(ArrowField[key].name).toBe(name);
}
