class Cls {
  method() {
    var _arguments = arguments;
    return babelHelpers.asyncToGenerator(function* () {
      () => {
        console.log(_arguments);
      };
    })();
  }
}
