const component = {
  value: 42,
  method() {
    var _this = this;
    return babelHelpers.asyncToGenerator(function* () {
      return _this.value;
    })();
  }
};
class Component {
  constructor() {
    this.value = 42;
  }
  method() {
    var _this2 = this;
    return babelHelpers.asyncToGenerator(function* () {
      return _this2.value;
    })();
  }
}
