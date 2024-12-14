// Just to make sure we're in sloppy mode. This is a syntax error in strict mode.
delete x;

class C {}

babelHelpers.defineProperty(C, "arrow", () => {
  if (true) {
    if (true) {
      {
        let f = function foo() {}
      }
    }
  }
  return () => {};
});

babelHelpers.defineProperty(C, "fn", function() {
  if (true) {
    if (true) {
      {
        let f = function foo() {}
      }
    }
  }
  return () => {};
});

babelHelpers.defineProperty(C, "arrowStrict", () => {
  "use strict";
  if (true) {}
  return () => {};
});

babelHelpers.defineProperty(C, "fnStrict", function() {
  "use strict";
  if (true) {}
  return () => {};
});

babelHelpers.defineProperty(C, "klass", class extends function() {} {
  constructor() {}
  method() {
    if (true) {}
    function foo() {}
  }
  [() => {}]() {}
});
