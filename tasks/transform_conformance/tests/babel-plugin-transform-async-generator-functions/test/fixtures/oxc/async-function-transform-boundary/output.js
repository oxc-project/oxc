function outer() {
  return async () => {
    void this;
    return eval("typeof _this");
  };
}
class Base {
  get value() {
    return 1;
  }
}
class Derived extends Base {
  field = async () => {};
  async direct() {
    super.value;
    return eval("typeof _superprop_getValue");
  }
  arrow() {
    return async () => {
      super.value;
      return eval("typeof _superprop_getValue");
    };
  }
  generator() {
    var _superprop_getValue = () => super.value;
    return babelHelpers.wrapAsyncGenerator(function* () {
      yield _superprop_getValue();
    })();
  }
}
