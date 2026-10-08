const read = () => A;
export class A {
  static { expect(() => read()).toThrow(ReferenceError); expect(A).toBe(this); }
}
expect(read()).toBe(A);
