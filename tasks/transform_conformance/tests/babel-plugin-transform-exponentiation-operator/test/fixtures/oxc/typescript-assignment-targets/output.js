import { receiver, replaceReceiver } from "receiver";

let key = 0;
function getObject() {
  return { x: 2 };
}
function getKey() {
  return "x";
}
function assignments(value) {
  var _unbound, _unbound2, _unbound3, _unbound4, _unbound5, _unbound6, _getObject, _getObject2, _getObject3, _getObject4, _getObject5, _getObject6, _getObject7, _key, _getObject8, _key2, _getObject9, _key3, _getObject10, _key4, _getObject11, _key5, _getObject12, _key6;
  value = Math.pow(value, 2);
  value = Math.pow(value, 2);
  value = Math.pow(value, 2);
  value = Math.pow(value, 2);
  value = Math.pow(value, 2);
  (value) = Math.pow(value, 2);
  _unbound = unbound, unbound = Math.pow(_unbound, 2);
  _unbound2 = unbound, unbound = Math.pow(_unbound2, 2);
  _unbound3 = unbound, unbound = Math.pow(_unbound3, 2);
  _unbound4 = unbound, unbound = Math.pow(_unbound4, 2);
  _unbound5 = unbound, unbound = Math.pow(_unbound5, 2);
  _unbound6 = unbound, (unbound) = Math.pow(_unbound6, 2);
  _getObject = getObject(), _getObject["x"] = Math.pow(_getObject["x"], 2);
  _getObject2 = getObject(), _getObject2["x"] = Math.pow(_getObject2["x"], 2);
  _getObject3 = getObject(), _getObject3["x"] = Math.pow(_getObject3["x"], 2);
  _getObject4 = getObject(), _getObject4["x"] = Math.pow(_getObject4["x"], 2);
  _getObject5 = getObject(), _getObject5["x"] = Math.pow(_getObject5["x"], 2);
  _getObject6 = getObject(), _getObject6["x"] = Math.pow(_getObject6["x"], 2);
  _getObject7 = getObject(), _key = ++key, _getObject7[_key] = Math.pow(_getObject7[_key], 2);
  _getObject8 = getObject(), _key2 = ++key, _getObject8[_key2] = Math.pow(_getObject8[_key2], 2);
  _getObject9 = getObject(), _key3 = ++key, _getObject9[_key3] = Math.pow(_getObject9[_key3], 2);
  _getObject10 = getObject(), _key4 = ++key, _getObject10[_key4] = Math.pow(_getObject10[_key4], 2);
  _getObject11 = getObject(), _key5 = ++key, _getObject11[_key5] = Math.pow(_getObject11[_key5], 2);
  _getObject12 = getObject(), _key6 = ++key, _getObject12[_key6] = Math.pow(_getObject12[_key6], 2);
}
class Base {
  get x() {
    return 2;
  }
  set x(value) {}
}
class Derived extends Base {
  assignments() {
    var _getKey, _getKey2, _getKey3, _getKey4, _getKey5, _getKey6;
    super["x"] = Math.pow(super["x"], 2);
    super["x"] = Math.pow(super["x"], 2);
    super["x"] = Math.pow(super["x"], 2);
    super["x"] = Math.pow(super["x"], 2);
    super["x"] = Math.pow(super["x"], 2);
    super["x"] = Math.pow(super["x"], 2);
    _getKey = getKey(), super[_getKey] = Math.pow(super[_getKey], 2);
    _getKey2 = getKey(), super[_getKey2] = Math.pow(super[_getKey2], 2);
    _getKey3 = getKey(), super[_getKey3] = Math.pow(super[_getKey3], 2);
    _getKey4 = getKey(), super[_getKey4] = Math.pow(super[_getKey4], 2);
    _getKey5 = getKey(), super[_getKey5] = Math.pow(super[_getKey5], 2);
    _getKey6 = getKey(), super[_getKey6] = Math.pow(super[_getKey6], 2);
  }
}
function reassignedReceiver() {
  var _obj, _ref;
  const original = { x: 2 }, replacement = { x: 3 };
  let obj = original;
  _obj = obj, _ref = (obj = replacement, "x"), _obj[_ref] = Math.pow(_obj[_ref], 2);
  return [original.x, replacement.x];
}
function reassignedReceiverInKeyCall() {
  var _obj2, _key7;
  const original = { x: 2 }, replacement = { x: 3 };
  let obj = original;
  function key() {
    obj = replacement;
    return "x";
  }
  _obj2 = obj, _key7 = key(), _obj2[_key7] = Math.pow(_obj2[_key7], 2);
  return [original.x, replacement.x];
}
function reassignedReceiverInEval() {
  var _obj3, _ref2;
  const original = { x: 2 }, replacement = { x: 3 };
  let obj = original;
  _obj3 = obj, _ref2 = (eval("obj = replacement"), "x"), _obj3[_ref2] = Math.pow(_obj3[_ref2], 2);
  return [original.x, replacement.x];
}
function importedReceiver() {
  var _receiver, _replaceReceiver;
  _receiver = receiver, _replaceReceiver = replaceReceiver(), _receiver[_replaceReceiver] = Math.pow(_receiver[_replaceReceiver], 2);
}
function literalKey(obj) {
  obj["x"] = Math.pow(obj["x"], 2);
}
