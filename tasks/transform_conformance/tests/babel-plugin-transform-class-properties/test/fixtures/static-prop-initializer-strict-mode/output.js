// Just to make sure we're in sloppy mode. This is a syntax error in strict mode.
delete x;

class C {}

babelHelpers.defineProperty(C, "arrow", { arrow: () => {
  if (true) {
    if (true) {
      {
        let f = function foo() {}
      }
    }
  }
  return () => {};
} }.arrow);

babelHelpers.defineProperty(C, "fn", function fn() {
  if (true) {
    if (true) {
      {
        let f = function foo() {}
      }
    }
  }
  return () => {};
});

babelHelpers.defineProperty(C, "arrowStrict", { arrowStrict: () => {
  "use strict";
  if (true) {}
  return () => {};
} }.arrowStrict);

babelHelpers.defineProperty(C, "fnStrict", function fnStrict() {
  "use strict";
  if (true) {}
  return () => {};
});

babelHelpers.defineProperty(C, "klass", class klass extends function() {} {
  constructor() {}
  method() {
    if (true) {}
    function foo() {}
  }
  [() => {}]() {}
});
