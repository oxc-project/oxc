var _staticBlock;
const read = () => A;
export let A = (class A {
  static #_ = _staticBlock = () => ((() => {
    expect(() => read()).toThrow(ReferenceError);
    expect(A).toBe(this);
  })(), this);
}, _staticBlock());
expect(read()).toBe(A);
