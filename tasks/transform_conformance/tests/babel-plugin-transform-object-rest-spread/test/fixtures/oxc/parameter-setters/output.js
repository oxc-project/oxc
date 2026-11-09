const _excluded = ["a", "b"], _excluded2 = ["a", "b"], _excluded3 = ["a", "b"];
const obj = { set value(_ref) {
  let { a = 1, b = a } = _ref, rest = babelHelpers.objectWithoutProperties(_ref, _excluded);
  this.result = b;
} };
class Setters {
  set value(_ref2) {
    let { a = 2, b = a } = _ref2, rest = babelHelpers.objectWithoutProperties(_ref2, _excluded2);
    this.result = b;
  }
  static set value([_ref3]) {
    let { a = 3, b = a } = _ref3, rest = babelHelpers.objectWithoutProperties(_ref3, _excluded3);
    this.result = b;
  }
}
