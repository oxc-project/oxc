function computed(object, value = (() => {
  var _object, _getKey;
  return _object = object, _getKey = getKey(), _object[_getKey] = Math.pow(_object[_getKey], 2);
})()) {}
function receiver(value = (() => {
  var _getObject;
  return _getObject = getObject(), _getObject["value"] = Math.pow(_getObject["value"], 2);
})()) {}
function nested(value = [(() => {
  var _getObject2, _getKey2;
  return _getObject2 = getObject(), _getKey2 = getKey(), _getObject2[_getKey2] = Math.pow(_getObject2[_getKey2], 2);
})()]) {}
function destructured({ value = (() => {
  var _getObject3;
  return _getObject3 = getObject(), _getObject3["value"] = Math.pow(_getObject3["value"], 2);
})() } = {}) {}
function childScope(value = (() => {
  var _getObject4, _getKey3;
  return _getObject4 = getObject(), _getKey3 = getKey(), _getObject4[_getKey3] = Math.pow(_getObject4[_getKey3], (() => value));
})()) {}
function nestedFunction(value = (() => {
  var _getObject5, _getKey4;
  return _getObject5 = getObject(), _getKey4 = getKey(), _getObject5[_getKey4] = Math.pow(_getObject5[_getKey4], 2);
})()) {}
function nestedParameters(value = ((other = (() => {
  var _getObject6;
  return _getObject6 = getObject(), _getObject6["value"] = Math.pow(_getObject6["value"], 2);
})()) => other)()) {}
const arrow = (object, value = (() => {
  var _object2, _getKey5;
  return _object2 = object, _getKey5 = getKey(), _object2[_getKey5] = Math.pow(_object2[_getKey5], 2);
})()) => value;
function noTemporary(object, value = object["value"] = Math.pow(object["value"], 2), other = object["x"] = Math.pow(object["x"], 2)) {}
function identifier(value = (() => {
  var _unbound;
  return _unbound = unbound, unbound = Math.pow(_unbound, 2);
})()) {}
function lexicalThis(value = (() => {
  var _this;
  return _this = this, _this["value"] = Math.pow(_this["value"], 2);
})()) {}
class Derived extends Base {
  method(value = (() => {
    var _getKey6;
    return _getKey6 = getKey(), super[_getKey6] = Math.pow(super[_getKey6], 2);
  })()) {}
  #value = 3;
  private(value = (() => {
    var _this2;
    return _this2 = this, _this2.#value = Math.pow(_this2.#value, 2);
  })()) {}
}
