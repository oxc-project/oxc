var _Counter;
var _value = /* @__PURE__ */ new WeakMap();
var _Counter_brand = /* @__PURE__ */ new WeakSet();
class Counter {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Counter_brand);
    babelHelpers.classPrivateFieldInitSpec(this, _value, 1);
  }
  add(k) {
    return babelHelpers.toSetter(_set_count.bind(babelHelpers.assertClassBrand(_Counter_brand, this)))._ = _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, this)) + k;
  }
  increment() {
    var _this$count;
    return babelHelpers.toSetter(_set_count.bind(babelHelpers.assertClassBrand(_Counter_brand, this)))._ = (_this$count = _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, this)), ++_this$count);
  }
  andAssign(value) {
    return _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, this)) && (babelHelpers.toSetter(_set_count.bind(babelHelpers.assertClassBrand(_Counter_brand, this)))._ = value);
  }
  nullishAssign(getObject, value) {
    var _getObject;
    return _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, _getObject = getObject())) ?? (babelHelpers.toSetter(_set_count.bind(babelHelpers.assertClassBrand(_Counter_brand, _getObject)))._ = value);
  }
  statement(k) {
    var _this$count2;
    _set_count.call(babelHelpers.assertClassBrand(_Counter_brand, this), _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, this)) + k);
    _set_count.call(babelHelpers.assertClassBrand(_Counter_brand, this), (_this$count2 = _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, this)), ++_this$count2));
    _get_count.call(babelHelpers.assertClassBrand(_Counter_brand, this)) || _set_count.call(babelHelpers.assertClassBrand(_Counter_brand, this), k);
  }
  static add(k) {
    return babelHelpers.toSetter(_set_sum.bind(babelHelpers.assertClassBrand(Counter, this)))._ = _get_sum.call(babelHelpers.assertClassBrand(Counter, this)) + k;
  }
  static addToClass(k) {
    return babelHelpers.toSetter(_set_sum.bind(babelHelpers.assertClassBrand(Counter, Counter)))._ = _get_sum.call(babelHelpers.assertClassBrand(Counter, Counter)) + k;
  }
}
_Counter = Counter;
function _get_count() {
  return babelHelpers.classPrivateFieldGet2(_value, this);
}
function _set_count(value) {
  babelHelpers.classPrivateFieldSet2(_value, this, value);
}
function _get_sum() {
  return babelHelpers.assertClassBrand(_Counter, this, _total)._;
}
function _set_sum(value) {
  _total._ = babelHelpers.assertClassBrand(_Counter, this, value);
}
var _total = { _: 1 };
