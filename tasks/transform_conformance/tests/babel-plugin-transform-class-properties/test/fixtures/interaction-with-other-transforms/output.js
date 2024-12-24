var _a, _e, _g;
let _ref, _ref2;

_ref = (_a = a) !== null && _a !== void 0 ? _a : b;
_ref2 = (_e = e) !== null && _e !== void 0 ? _e : f;
class C {
  constructor() {
    var _c;
    babelHelpers.defineProperty(this, _ref, (_c = c) !== null && _c !== void 0 ? _c : d);
  }
}
babelHelpers.defineProperty(C, _ref2, (_g = g) !== null && _g !== void 0 ? _g : h);
(() => {
  var _i;
  (_i = i) !== null && _i !== void 0 ? _i : j;
})();

class C2 extends S {
  constructor() {
    var _super = (..._args) => {
      var _k;
      return super(..._args), babelHelpers.defineProperty(this, "prop", (_k = k) !== null && _k !== void 0 ? _k : l), this;
    };
    if (true) {
      _super();
    }
  }
}
