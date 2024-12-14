function createClassDeclaration() {
  class C {
    [this] = 1;
    [this + 'bar'] = 2;
  }
  return C;
}

function createClassExpression() {
  return class {
    [this] = 3;
    [this + 'bar'] = 4;
  };
}

const C = createClassDeclaration.call("foo");
expect(new C().foo).toBe(1);
expect(new C().foobar).toBe(2);

const D = createClassExpression.call("foo");
expect(new D().foo).toBe(3);
expect(new D().foobar).toBe(4);
