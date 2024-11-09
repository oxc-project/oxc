const Obj2 = {
  foo() {
    return babelHelpers.asyncToGenerator(function* () {
      console.log(log);
    })();
  }
};