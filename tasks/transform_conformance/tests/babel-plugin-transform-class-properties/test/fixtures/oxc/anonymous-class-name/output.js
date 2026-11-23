var _Class2, _Class3, _Class4, _Class5, _Class_brand;
function make(v) {
  var _Class;
  return _Class = (0, class extends v {}), babelHelpers.defineProperty(_Class, "r", 1), _Class;
}
const obj = {
  x: (_Class2 = class x {}, babelHelpers.defineProperty(_Class2, "y", 1), _Class2),
  "a-b": (_Class3 = { "a-b": class {} }["a-b"], babelHelpers.defineProperty(_Class3, "y", 1), _Class3)
};
const C = (_Class4 = class C {}, babelHelpers.defineProperty(_Class4, "y", 1), _Class4);
let D;
D = (_Class5 = { D: class {
  m() {
    return D;
  }
} }.D, babelHelpers.defineProperty(_Class5, "y", 1), _Class5);
const H = (_Class_brand = /* @__PURE__ */ new WeakSet(), class H {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Class_brand);
  }
});
function _m() {}
