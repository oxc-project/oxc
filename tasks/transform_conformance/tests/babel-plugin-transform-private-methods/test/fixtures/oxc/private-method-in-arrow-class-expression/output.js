const makeClass = () => {
  var _field, _Class_brand, _read;
  return _field = /* @__PURE__ */ new WeakMap(), _Class_brand = /* @__PURE__ */ new WeakSet(), _read = function() {
    return babelHelpers.classPrivateFieldGet2(_field, this);
  }, class {
    constructor() {
      babelHelpers.classPrivateMethodInitSpec(this, _Class_brand);
      babelHelpers.classPrivateFieldInitSpec(this, _field, 41);
    }
    run() {
      return babelHelpers.assertClassBrand(_Class_brand, this, _read).call(this) + 1;
    }
  };
};
export function get() {
  return new (makeClass())().run();
}
