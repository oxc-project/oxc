class Outer {
  constructor() {
    var _this = this;
    babelHelpers.asyncToGenerator(function* () {
      return [_this, 2];
    });

    class Inner extends Outer {
      constructor() {
        var _this2;

        if (condition) {
          const _super = super();
          _this2 = this;
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
  }
}
