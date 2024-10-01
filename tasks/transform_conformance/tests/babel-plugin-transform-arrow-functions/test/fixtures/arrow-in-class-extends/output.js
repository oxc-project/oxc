var _this = this;

let f;

class C extends (
  f = function() { return _this; },
  class {}
) {}

function outer() {
  var _this2 = this;
  class C extends (
    f = function() { return _this2; },
    class {}
  ) {}
}
