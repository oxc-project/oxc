const classes = [];
for (let i = 0; i < 2; i++) {
  class C {
    static has(C = this) {
      return babelHelpers.checkInRHS(C) === _C;
    }
    static read(C = this) {
      return babelHelpers.assertClassBrand(_C, C, _m).call(C);
    }
  }
  const _C = C;
  function _m() {
    return 42;
  }
  classes.push(C);
}
