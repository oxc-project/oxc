var _this = this;

let f;

class C {
  [f = function() { return _this; }] = 1;
}

function outer() {
  var _this2 = this;
  class C {
    [f = function() { return _this2; }] = 1;
  }
}
