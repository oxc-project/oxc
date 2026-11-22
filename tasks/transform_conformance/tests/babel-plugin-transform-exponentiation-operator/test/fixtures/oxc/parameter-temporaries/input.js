function computed(object, value = object[getKey()] **= 2) {}
function receiver(value = getObject().value **= 2) {}
function nested(value = [getObject()[getKey()] **= 2]) {}
function destructured({ value = getObject().value **= 2 } = {}) {}
function childScope(value = getObject()[getKey()] **= (() => value)) {}
function nestedFunction(value = (() => {
  return getObject()[getKey()] **= 2;
})()) {}
function nestedParameters(value = ((other = getObject().value **= 2) => other)()) {}
const arrow = (object, value = object[getKey()] **= 2) => value;
function noTemporary(object, value = object.value **= 2, other = object["x"] **= 2) {}
function identifier(value = unbound **= 2) {}
function lexicalThis(value = this.value **= 2) {}
class Derived extends Base {
  method(value = super[getKey()] **= 2) {}
  #value = 3;
  private(value = this.#value **= 2) {}
}
