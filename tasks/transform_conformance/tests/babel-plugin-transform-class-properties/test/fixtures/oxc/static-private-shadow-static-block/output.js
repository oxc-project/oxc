let value;
let _C;
class C {
  static has(C = this) {
    return babelHelpers.checkInRHS(C) === _C;
  }
  static read(C = this) {
    return babelHelpers.assertClassBrand(_C, C, _m).call(C);
  }
  static {
    _C = this;
    value = [
      this.has(),
      this.read(),
      this.has({})
    ];
  }
}
function _m() {
  return 42;
}
