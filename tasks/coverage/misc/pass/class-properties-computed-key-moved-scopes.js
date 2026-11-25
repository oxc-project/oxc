// Computed keys of class elements which are moved out of the class must have their scopes
// re-parented, and lose strict mode, because this file is a sloppy-mode script.

class A {
  [() => {}] = 1;
  static [function () {}] = 2;
  [(() => {})()]() {}
  accessor [function () {}] = 3;
  static [function () { "use strict"; }] = 4;
  [function () { return () => {}; }] = 5;
  static [function () { "use strict"; return () => {}; }] = 6;
  [class { m() { return () => {}; } }] = 7;
}

let B = class {
  [() => {}] = 1;
  static [function () {}] = 2;
  [(() => {})()]() {}
  accessor [function () {}] = 3;
  static [function () { "use strict"; }] = 4;
  [function () { return () => {}; }] = 5;
  static [function () { "use strict"; return () => {}; }] = 6;
  [class { m() { return () => {}; } }] = 7;
};

let f = () => class {
  [() => {}] = 1;
  static [function () {}] = 2;
};

function g() {
  "use strict";
  let C = class {
    [() => {}] = 1;
    static [function () {}] = 2;
  };
}
