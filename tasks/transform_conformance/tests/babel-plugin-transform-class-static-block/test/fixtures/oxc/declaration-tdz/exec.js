const read = () => A;
let observed;
class A {
  static {
    expect(A).toBe(this);
    try { read(); observed = "initialized"; } catch (error) { observed = error.name; }
  }
}
expect(observed).toBe("ReferenceError");
expect(read()).toBe(A);
const Original = A;
A = 42;
expect(A).toBe(42);
expect(typeof Original).toBe("function");
let Saved;
let readThrowing;
try {
  readThrowing = () => Throwing;
  class Throwing {
    static { Saved = this; throw new Error("stop"); }
  }
} catch {}
expect(typeof Saved).toBe("function");
expect(readThrowing).toThrow(ReferenceError);
class Self {
  static getSelf() { return Self; }
  static { expect(() => { Self = null; }).toThrow(TypeError); }
}
const OriginalSelf = Self;
Self = null;
expect(OriginalSelf.getSelf()).toBe(OriginalSelf);
