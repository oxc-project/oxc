class Foo {
  accessor a = 1;
  static accessor b = 2;
  accessor ["c"] = 3;
  accessor #d = 4;
  getD() { return this.#d; }
  setD(v) { this.#d = v; }
}

const foo = new Foo();
expect([foo.a, Foo.b, foo.c, foo.getD()]).toEqual([1, 2, 3, 4]);
foo.a = 10;
foo.setD(40);
expect([foo.a, foo.getD()]).toEqual([10, 40]);
class Sub extends Foo {}
expect(() => Sub.b).toThrow(TypeError);
