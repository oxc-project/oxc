let f;

class S {}

class C extends S {
  constructor(x) {
    var _this;
    super(
      (super(), _this = this),
      this.x = x,
      f = babelHelpers.asyncToGenerator(function* () {
        return _this;
      })
    );
    _this = this;
  }
}
