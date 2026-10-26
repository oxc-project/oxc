let _key;
const key = Symbol("method");
class A {
  static [_key = babelHelpers.toPropertyKey(key)] = ((() => {})(), babelHelpers.setFunctionName(function() {}, _key));
  static #f = ((() => {})(), babelHelpers.setFunctionName(() => {}, "#f"));
  static getName() {
    return this.#f.name;
  }
}
