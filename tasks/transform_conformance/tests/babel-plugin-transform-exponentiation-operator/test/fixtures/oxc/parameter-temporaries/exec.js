let depth = 0;
const outer = { x: 3 };
const inner = { x: 5 };
function key() {
  if (depth++ === 0) recursive(inner);
  return "x";
}
function recursive(object, value = object[key()] **= 2) {
  return value;
}
expect(recursive(outer)).toBe(9);
expect([outer.x, inner.x, depth]).toEqual([9, 25, 2]);

class Base {
  getObject() {
    return this.object;
  }
}
class Derived extends Base {
  constructor() {
    super();
    this.object = { x: 3 };
  }
  method(key, value = super.getObject()[key] **= arguments.length + this.offset) {
    return value;
  }
}
const instance = new Derived();
instance.offset = 1;
expect(instance.method("x")).toBe(9);
expect(instance.object.x).toBe(9);

function Constructor(object, value = object[new.target.name] **= 2) {
  this.value = value;
}
const object = { Constructor: 3 };
expect(new Constructor(object).value).toBe(9);
expect(object.Constructor).toBe(9);

const order = [];
const observed = {
  get value() { order.push("get"); return 3; },
  set value(value) { order.push(["set", value]); },
};
function getObserved() { order.push("object"); return observed; }
function getKey() { order.push("key"); return "value"; }
function exponent() { order.push("right"); return 2; }
function ordered(value = getObserved()[getKey()] **= exponent()) { return value; }
expect(ordered()).toBe(9);
expect(order).toEqual(["object", "key", "get", "right", ["set", 9]]);

function tdz(value = observed[later] **= 2, later = "value") {}
expect(() => tdz()).toThrow(ReferenceError);
const scopeObject = { value: 3 };
function parameterScope(value = scopeObject[getKey()] **= 2) {
  const scopeObject = null;
  return value;
}
expect(parameterScope()).toBe(9);
