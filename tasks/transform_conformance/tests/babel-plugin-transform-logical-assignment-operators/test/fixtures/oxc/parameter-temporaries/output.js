function receiver(value = (() => {
  var _getObject;
  return (_getObject = getObject()).value || (_getObject.value = 1);
})()) {}
function key(object, value = (() => {
  var _getKey;
  return object[_getKey = getKey()] && (object[_getKey] = 2);
})()) {}
function receiverAndKey(value = (() => {
  var _getObject2, _getKey2;
  return (_getObject2 = getObject())[_getKey2 = getKey()] ?? (_getObject2[_getKey2] = 1);
})()) {}
function nested(value = [(() => {
  var _getObject3;
  return (_getObject3 = getObject()).value || (_getObject3.value = 1);
})()]) {}
function destructured({ value = (() => {
  var _getObject4;
  return (_getObject4 = getObject()).value || (_getObject4.value = 1);
})() } = {}) {}
function mutated(object, value = (() => {
  var _object;
  return (_object = object).value || (_object.value = 1);
})()) {
  object = null;
}
function childScope(value = (() => {
  var _getObject5;
  return (_getObject5 = getObject()).value || (_getObject5.value = () => value);
})()) {}
function nestedFunction(value = (() => {
  var _getObject6;
  return (_getObject6 = getObject()).value || (_getObject6.value = 1);
})()) {}
function nestedParameters(value = ((other = (() => {
  var _getObject7;
  return (_getObject7 = getObject()).value || (_getObject7.value = 1);
})()) => other)()) {}
const arrow = (value = (() => {
  var _getObject8;
  return (_getObject8 = getObject()).value || (_getObject8.value = 1);
})()) => value;
function noTemporary(object, key, value = object.value || (object.value = 1), other = object[key] && (object[key] = 2)) {}
function identifier(value = object || (object = 1)) {}
function lexicalThis(value = this.value || (this.value = 1)) {}
function literal(value = "object".value || ("object".value = 1), other = `object`["key"] && (`object`["key"] = 2)) {}
class Derived extends Base {
  method(value = super.value || (super.value = 1)) {}
}

