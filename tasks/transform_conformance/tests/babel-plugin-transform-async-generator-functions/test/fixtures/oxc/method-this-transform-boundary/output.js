const component = {
  value: 42,
  async method() {
    this.value;
    return eval("typeof _this");
  },
  generatorMethod() {
    var _this = this;
    return babelHelpers.wrapAsyncGenerator(function* () {
      yield _this.value;
    })();
  }
};
