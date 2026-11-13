function receiver(value = (getObject().value ||= 1)) {}
function key(object, value = (object[getKey()] &&= 2)) {}
function receiverAndKey(value = (getObject()[getKey()] ??= 1)) {}
function nested(value = [getObject().value ||= 1]) {}
function destructured({ value = (getObject().value ||= 1) } = {}) {}
function mutated(object, value = (object.value ||= 1)) {
  object = null;
}
function childScope(value = (getObject().value ||= () => value)) {}
function nestedFunction(value = (() => {
  return getObject().value ||= 1;
})()) {}
function nestedParameters(value = ((other = (getObject().value ||= 1)) => other)()) {}
const arrow = (value = (getObject().value ||= 1)) => value;

function noTemporary(object, key, value = (object.value ||= 1), other = (object[key] &&= 2)) {}
function identifier(value = (object ||= 1)) {}
function lexicalThis(value = (this.value ||= 1)) {}
function literal(value = ("object".value ||= 1), other = (`object`["key"] &&= 2)) {}
class Derived extends Base {
  method(value = (super.value ||= 1)) {}
}
