var _privateField = new WeakMap();
var _Cl_brand = new WeakSet();
class Cl {
  get self() {
    this.counter++;
    return this;
  }
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Cl_brand);
    babelHelpers.classPrivateFieldInitSpec(this, _privateField, 0);
    babelHelpers.defineProperty(this, "counter", 0);
    this.self, 1([(this.self, babelHelpers.readOnlyError("#privateFieldValue"))._] = [1]), babelHelpers.readOnlyError("#privateFieldValue");
  }
}
function _get_privateFieldValue() {
  return babelHelpers.classPrivateFieldGet2(_privateField, this);
}
const cl = new Cl();
