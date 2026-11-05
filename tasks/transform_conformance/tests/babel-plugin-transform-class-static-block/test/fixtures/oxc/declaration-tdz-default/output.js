var _staticBlock;
const read = () => A;
let A = (class A {
  static #_ = _staticBlock = () => ((() => {
    expect(() => read()).toThrow(ReferenceError);
    expect(A).toBe(this);
  })(), this);
}, _staticBlock());
export { A as default };
expect(read()).toBe(A);
