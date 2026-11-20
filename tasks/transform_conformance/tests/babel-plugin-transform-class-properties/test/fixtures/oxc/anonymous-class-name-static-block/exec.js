function make(v) {
  return class extends v {
    static {
      this.n = this.name;
    }
  };
}
expect(make(class {}).name).toBe("");
expect(make(class {}).n).toBe("");

function id(c) {
  return c;
}
expect(id(class { static {} }).name).toBe("");

const C = class {
  static {
    this.n = this.name;
  }
};
expect(C.name).toBe("C");
expect(C.n).toBe("C");

const obj = {
  x: class {
    static {
      this.n = this.name;
    }
  },
};
expect(obj.x.name).toBe("x");
expect(obj.x.n).toBe("x");
