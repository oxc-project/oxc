const dec = () => {};

class Outer {
  #outer = 1;
  @dec method() {
    return class Inner {
      #inner = 2;
      get(o) {
        return this.#inner + o.#outer;
      }
    };
  }
}

const Inner = new Outer().method();
expect(new Inner().get(new Outer())).toBe(3);
