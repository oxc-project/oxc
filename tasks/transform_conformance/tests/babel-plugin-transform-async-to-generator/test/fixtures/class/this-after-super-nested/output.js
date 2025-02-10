class Outer {
  constructor() {
    var _this = this;
    babelHelpers.asyncToGenerator(function* () {
      return [_this, 1];
    });

    class Inner extends Outer {
      constructor() {
        var _this2;

        if (condition) {
          const _super = (super(), _this2 = this);
          this.fn = babelHelpers.asyncToGenerator(function* () {
            return [_this2, 2];
          });
        }

        super();
        _this2 = this;
        babelHelpers.asyncToGenerator(function* () {
          return [_this2, 3];
        });
      }
    }
  }
}

class Outer2 {
  constructor() {
    var _this3 = this;
    babelHelpers.asyncToGenerator(function* () {
      return [_this3, 4];
    });

    class Inner extends Outer2 {
      constructor() {
        var _this4;

        if (condition) {
          const _super = (super(), _this4 = this);
          this.fn = babelHelpers.asyncToGenerator(function* () {
            return [_this4, 5];
          });
        }

        super();
        _this4 = this;
        babelHelpers.asyncToGenerator(function* () {
          return [_this4, 6];
        });
      }
    }
  }
}
