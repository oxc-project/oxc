function createClassDeclaration() {
  let _this, _ref;
  _this = this;
  _ref = this + "bar";
  class C {
    constructor() {
      babelHelpers.defineProperty(this, _this, 1);
      babelHelpers.defineProperty(this, _ref, 2);
    }
  }
  return C;
}

function createClassExpression() {
  let _this2, _ref2;
  return _this2 = this, _ref2 = this + "bar", class {
    constructor() {
      babelHelpers.defineProperty(this, _this2, 3);
      babelHelpers.defineProperty(this, _ref2, 4);
    }
  };
}

const C = createClassDeclaration.call("foo");
expect(new C().foo).toBe(1);
expect(new C().foobar).toBe(2);

const D = createClassExpression.call("foo");
expect(new D().foo).toBe(3);
expect(new D().foobar).toBe(4);
