var _fn = /* @__PURE__ */ new WeakMap();
class C {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _fn, { "#fn": function() {
      return this;
    } }["#fn"]);
  }
  run() {
    return (this === null || this === void 0 ? void 0 : babelHelpers.classPrivateFieldGet2(_fn, this).bind(this))();
  }
  runOptional() {
    return (this === null || this === void 0 ? void 0 : babelHelpers.classPrivateFieldGet2(_fn, this)?.bind(this))?.();
  }
  runMultiple() {
    return (this === null || this === void 0 ? void 0 : babelHelpers.classPrivateFieldGet2(_fn, this).bind(this))();
  }
  runOptionalMultiple() {
    return (this === null || this === void 0 ? void 0 : babelHelpers.classPrivateFieldGet2(_fn, this)?.bind(this))?.();
  }
}
