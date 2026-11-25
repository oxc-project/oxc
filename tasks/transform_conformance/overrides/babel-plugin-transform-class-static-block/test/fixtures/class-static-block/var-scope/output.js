var _staticBlock;
let res = [];
let foo = 1;
let A = (class A {
  static x = ((_value) => (_staticBlock = () => ((() => {
    var foo = 4;
  })(), res.push(foo), this), _value))(((() => {
    var foo = 3;
  })(), res.push(foo), void 0));
}, _staticBlock());
expect(res).toEqual([1, 1]);
