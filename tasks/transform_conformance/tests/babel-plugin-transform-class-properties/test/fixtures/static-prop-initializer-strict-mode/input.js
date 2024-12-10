// Just to make sure we're in sloppy mode. This is a syntax error in strict mode.
delete x;

class C {
  static arrow = () => {
    if (true) {
      if (true) {
        {
          let f = function foo() {};
        }
      }
    }
    return () => {};
  };

  static fn = function() {
    if (true) {
      if (true) {
        {
          let f = function foo() {}
        }
      }
    }
    return () => {};
  };

  static arrowStrict = () => {
    "use strict";
    if (true) {}
    return () => {};
  };

  static fnStrict = function() {
    "use strict";
    if (true) {}
    return () => {};
  };

  static klass = class extends function() {} {
    constructor() {}
    method() {
      if (true) {}
      function foo() {}
    }
    [() => {}]() {}
  };
}
