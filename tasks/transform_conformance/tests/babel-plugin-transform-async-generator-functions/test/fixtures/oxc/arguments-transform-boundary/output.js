const object = {
  async method() {
    return arguments;
  },
  generatorMethod() {
    var _arguments = arguments;
    return babelHelpers.wrapAsyncGenerator(function* () {
      return _arguments;
    })();
  }
};
class Example {
  async method() {
    return arguments;
  }
  generatorMethod() {
    var _arguments2 = arguments;
    return babelHelpers.wrapAsyncGenerator(function* () {
      return _arguments2;
    })();
  }
}
function outer() {
  return async () => arguments;
}
