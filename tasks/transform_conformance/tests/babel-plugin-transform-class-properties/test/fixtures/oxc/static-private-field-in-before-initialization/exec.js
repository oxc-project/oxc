const values = [];
let calls = 0;

class Foo {
  static {
    values.push(#b in this);
  }
  static #b = 0;
  static {
    values.push(#b in this);
  }
}

class Bar {
  static {
    values.push(#b in (calls++, this));
  }
  static #b;
  static {
    values.push(#b in this);
  }
}

expect(values).toEqual([false, true, false, true]);
expect(calls).toBe(1);

expect(() => {
  class Throws {
    static {
      #b in null;
    }
    static #b;
  }
}).toThrow(TypeError);
