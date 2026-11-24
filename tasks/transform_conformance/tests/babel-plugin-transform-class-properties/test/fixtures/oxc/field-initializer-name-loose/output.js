var _Class3, _Class4, _Class5;
let _k, _k2, _key, _key2, _k3, _k4, _k5;
var _w = /* @__PURE__ */ babelHelpers.classPrivateFieldLooseKey("w");
var _sw = /* @__PURE__ */ babelHelpers.classPrivateFieldLooseKey("sw");
_k = babelHelpers.toPropertyKey(k);
_k2 = babelHelpers.toPropertyKey(k);
_key = babelHelpers.toPropertyKey(key());
_key2 = babelHelpers.toPropertyKey(key());
_k3 = babelHelpers.toPropertyKey(k2);
_k4 = babelHelpers.toPropertyKey(k2);
_k5 = babelHelpers.toPropertyKey(k3);
class F {
  constructor() {
    var _Class, _Class2;
    this.x = class x {};
    Object.defineProperty(this, _w, {
      writable: true,
      value: { "#w": class {} }["#w"]
    });
    this[1] = { "1": class {} }["1"];
    this.fn = function fn() {};
    this.arrow = { arrow: () => {} }.arrow;
    this.paren = class paren {};
    this.shadowed = { shadowed: function() {
      return shadowed;
    } }.shadowed;
    this[_k] = babelHelpers.setFunctionName(class {}, _k);
    this[_key] = babelHelpers.setFunctionName(function() {}, _key);
    this.inst = (_Class = class inst {}, _Class.n = _Class.name, _Class);
    this[_k3] = (_Class2 = babelHelpers.setFunctionName(class {}, _k3), _Class2.n = _Class2.name, _Class2);
  }
}
F.y = class y {};
Object.defineProperty(F, _sw, {
  writable: true,
  value: { "#sw": class {} }["#sw"]
});
F[_k2] = babelHelpers.setFunctionName(() => {}, _k2);
F[_key2] = babelHelpers.setFunctionName(class {}, _key2);
F.stat = (_Class3 = class stat {}, _Class3.n = _Class3.name, _Class3);
F[_k4] = (_Class4 = babelHelpers.setFunctionName(class {}, _k4), _Class4.n = _Class4.name, _Class4);
F[_k5] = (_Class5 = babelHelpers.setFunctionName(class {}, _k5), babelHelpers.defineProperty(_Class5, "name", "custom"), _Class5);
