let _k;
_k = babelHelpers.toPropertyKey(k);
class F {
  constructor() {
    babelHelpers.defineProperty(this, "x", class x {});
    babelHelpers.defineProperty(this, "y", { y: (() => {}) }.y);
    babelHelpers.defineProperty(this, "z", function z() {});
    babelHelpers.defineProperty(this, _k, babelHelpers.setFunctionName(class {}, _k));
  }
}
