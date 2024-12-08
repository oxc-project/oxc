function sloppy() {
  class C extends S {
    prop = 1;
    constructor(x = super()) {}
  }
}

function strict() {
  "use strict";
  class C extends S {
    prop = 1;
    constructor(x = super()) {}
  }
}
