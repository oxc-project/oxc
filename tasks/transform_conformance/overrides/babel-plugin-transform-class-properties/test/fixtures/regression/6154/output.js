class Test {
  constructor() {
    var _Other;
    class Other extends Test {
      constructor(..._args) {
        super(..._args);
        babelHelpers.defineProperty(this, "a", () => super.test);
      }
    }
    _Other = Other;
    babelHelpers.defineProperty(Other, "a", () => babelHelpers.superPropGet(_Other, "test", _Other));
  }
}
