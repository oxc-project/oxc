const classes: [any, any][] = [];
function dec(Class: any): any {
  const Decorated = class extends Class {};
  classes.push([Class, Decorated]);
  return Decorated;
}

@dec
class Foo {
  static #p = 1;
  static #m() {
    return 2;
  }
  static fromInitializer = this.#p;
  static fromBlock: number;
  static {
    this.fromBlock = Foo.#m();
  }
  static getP() {
    return this.#p;
  }
  static callM() {
    return this.#m();
  }
  static hasP(obj: any) {
    return #p in obj;
  }
  static self() {
    return Foo;
  }
}

@dec
export default class {
  static #p = 3;
  static getP() {
    return this.#p;
  }
}

const [[OriginalFoo], [OriginalDefault, Default]] = classes;

// References to the class name see the decorated class.
expect(Foo).not.toBe(OriginalFoo);
expect(Foo.self()).toBe(Foo);
expect(Foo.fromInitializer).toBe(1);
expect(Foo.fromBlock).toBe(2);

// Static private brand checks are against the original class.
expect(() => Foo.getP()).toThrow(TypeError);
expect(() => Foo.callM()).toThrow(TypeError);
expect(Foo.getP.call(OriginalFoo)).toBe(1);
expect(Foo.callM.call(OriginalFoo)).toBe(2);
expect(Foo.hasP(Foo)).toBe(false);
expect(Foo.hasP(OriginalFoo)).toBe(true);

expect(() => Default.getP()).toThrow(TypeError);
expect(Default.getP.call(OriginalDefault)).toBe(3);
