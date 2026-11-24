function make(v) {
  return class extends v { static r = 1 };
}
const obj = { x: class { static y = 1 }, "a-b": class { static y = 1 } };
const C = class { static y = 1 };
let D;
D = class {
  static y = 1;
  m() {
    return D;
  }
};
const H = class { #m() {} };
