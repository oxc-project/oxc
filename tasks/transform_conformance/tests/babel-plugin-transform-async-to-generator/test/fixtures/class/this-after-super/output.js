class C extends S {
  constructor() {
    var _this;
    if (condition) {
      const _super = super();
      _this = this;
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
