function sloppy() {
  let _super = function() {
    "use strict";
    babelHelpers.defineProperty(this, "prop", 1);
    return this;
  };
  class C extends S {
    constructor(x = _super.call(super())) {}
  }
}

function strict() {
  "use strict";
  let _super2 = function() {
    babelHelpers.defineProperty(this, "prop", 1);
    return this;
  };
  class C extends S {
    constructor(x = _super2.call(super())) {}
  }
}
