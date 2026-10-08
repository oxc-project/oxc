var _staticBlock, _staticBlock2, _staticBlock3;
const calls = [];
let Before = (class Before {
  static #_ = _staticBlock = () => (calls.push("before"), this);
}, _staticBlock());
let After = (class After {
  static #_ = _staticBlock2 = () => (calls.push("after"), this);
}, _staticBlock2());
let Mixed = (class Mixed {
  static present = ((_value) => (_staticBlock3 = () => (calls.push("last"), this), _value))((calls.push("first"), calls.push("field"), 1));
}, _staticBlock3());
