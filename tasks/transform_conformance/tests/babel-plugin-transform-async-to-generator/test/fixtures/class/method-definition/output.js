class ClassWithAsyncMethods {
  with_parameters(p, [p1, p2]) {
    return babelHelpers.asyncToGenerator(function* () {
      return console.log(p, p1, p2);
    })();
  }
  without_parameters() {
    return babelHelpers.asyncToGenerator(function* () {
      console.log(ClassWithAsyncMethods);
    })();
  }
}