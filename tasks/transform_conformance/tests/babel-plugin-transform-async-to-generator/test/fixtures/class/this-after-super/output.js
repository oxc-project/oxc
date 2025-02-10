class S {}

class C extends S {
  constructor() {
    var _this;
    if (true) {
      const _super = (super(), _this = this);
      this.fn = babelHelpers.asyncToGenerator(function* () {
        return [_this, 1];
      });
    }

    super();
    _this = this;
    babelHelpers.asyncToGenerator(function* () {
      return [_this, 2];
    });
  }
}

class C2 extends S {
  constructor() {
    var _this2;
    if (true) {
      const _super = (super(), _this2 = this);
      this.fn = babelHelpers.asyncToGenerator(function* () {
        return [_this2, 1];
      });
    }

    super();
    _this2 = this;
    babelHelpers.asyncToGenerator(function* () {
      return [_this2, 2];
    });
  }
}
