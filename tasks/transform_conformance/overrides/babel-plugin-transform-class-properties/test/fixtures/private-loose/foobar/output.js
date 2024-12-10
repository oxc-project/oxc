var _scopedFunctionWithThis = /*#__PURE__*/babelHelpers.classPrivateFieldLooseKey("scopedFunctionWithThis");
class Child extends Parent {
  constructor() {
    super();
    Object.defineProperty(this, _scopedFunctionWithThis, {
      writable: true,
      value: () => {
        this.name = {};
      }
    });
  }
}
