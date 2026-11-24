class F {
  x = class {};
  static y = class {};
  #w = class {};
  static #sw = class {};
  1 = class {};
  fn = function () {};
  arrow = () => {};
  paren = (class {});
  shadowed = function () {
    return shadowed;
  };
  [k] = class {};
  static [k] = () => {};
  [key()] = function () {};
  static [key()] = class {};
  inst = class { static n = this.name };
  static stat = class { static n = this.name };
  [k2] = class { static n = this.name };
  static [k2] = class { static n = this.name };
  static [k3] = class { static name = "custom" };
}
