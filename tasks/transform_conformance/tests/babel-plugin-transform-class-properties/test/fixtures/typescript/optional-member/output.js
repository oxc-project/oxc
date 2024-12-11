class X {
  method() {
    const o = {
      X
    };
    babelHelpers.assertClassBrand(X, o.X, _a)._;
    babelHelpers.assertClassBrand(X, o.X, _a)._;
    babelHelpers.assertClassBrand(X, o.X, _a)._;
  }
}
var _a = {
  _: 0
};