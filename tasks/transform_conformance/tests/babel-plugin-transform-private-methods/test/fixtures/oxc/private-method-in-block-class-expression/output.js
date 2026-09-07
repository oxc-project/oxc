function makeClass(value) {
  {
    var _Class_brand, _read;
    const captured = value;
    return _Class_brand = /* @__PURE__ */ new WeakSet(), _read = function* () {
      yield captured;
    }, class {
      constructor() {
        babelHelpers.classPrivateMethodInitSpec(this, _Class_brand);
      }
      run() {
        return babelHelpers.assertClassBrand(_Class_brand, this, _read).call(this).next().value + 1;
      }
    };
  }
}
export function get() {
  return new (makeClass(41))().run();
}
