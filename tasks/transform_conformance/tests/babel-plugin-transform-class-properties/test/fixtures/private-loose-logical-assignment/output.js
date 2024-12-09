var _nullish = babelHelpers.classPrivateFieldLooseKey("nullish");
var _and = babelHelpers.classPrivateFieldLooseKey("and");
var _or = babelHelpers.classPrivateFieldLooseKey("or");
class Foo {
  constructor() {
    Object.defineProperty(this, _nullish, {
      writable: true,
      value: 0
    });
    Object.defineProperty(this, _and, {
      writable: true,
      value: 0
    });
    Object.defineProperty(this, _or, {
      writable: true,
      value: 0
    });
  }
  self() {
    return this;
  }
  test() {
    babelHelpers.classPrivateFieldLooseBase(this, _nullish)[_nullish] ??= 42;
    babelHelpers.classPrivateFieldLooseBase(this, _and)[_and] &&= 0;
    babelHelpers.classPrivateFieldLooseBase(this, _or)[_or] ||= 0;
    babelHelpers.classPrivateFieldLooseBase(this.self(), _nullish)[_nullish] ??= 42;
  }
}
